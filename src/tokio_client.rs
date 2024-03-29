use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use tokio::select;
use tokio::sync::{mpsc, watch};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Framed};
use tracing::{debug, error, Instrument};

use crate::proto;

pub fn connect(path: &str) -> anyhow::Result<(RefereeClientReader, RefereeClientWriter)> {
    let serial_stream = tokio_serial::new(path, 115200).open_native_async()?;
    let framed = codec::RefereeCodec.framed(serial_stream);
    let (sink, stream) = framed.split();
    let (client, writer) = (RefereeClientReader { stream }, RefereeClientWriter { sink, seq: 0 });
    Ok((client, writer))
}

pub struct RefereeClientWriter {
    sink: SplitSink<Framed<SerialStream, codec::RefereeCodec>, proto::Frame2>,
    seq: u8,
}

impl RefereeClientWriter {
    pub async fn send_message_with_known_data_length(
        &mut self,
        message: proto::Message, data_length: u16,
    ) -> Result<(), codec::RefereeCodecError> {
        let frame = proto::Frame2 {
            data_length,
            seq: self.seq,
            crc8: 0,
            message,
            crc_frame_tail: 0,
        };
        self.seq = self.seq.wrapping_add(1);
        debug!("Sending frame: {:?}", frame);
        self.sink.send(frame).await
    }

    /// 由雷达站发送给裁判系统，用于告知己方操作手对方机器人的位置
    ///
    /// 不论雷达站属于哪一方，都以小地图左下角（红方补给区）为原点，双方对向为第一维，范围为 `(0.0..28.0, 0.0..15.0)`
    pub async fn send_minimap_receipt(&mut self, target_robot_id: u16, target_position: (f32, f32)) -> Result<(), codec::RefereeCodecError> {
        self.send_message_with_known_data_length(proto::Message::MinimapReceipt {
            target_robot_id,
            target_position,
        }, 10).await
    }

    /// 机器人之间通信
    pub async fn send_p2p(&mut self, content_id: u16, send_id: u16, receive_id: u16, content: Vec<u8>) -> Result<(), codec::RefereeCodecError> {
        let data_length = (content.len() + 6) as u16;
        self.send_message_with_known_data_length(proto::Message::StudentInteractiveData(
            proto::StudentInteractiveData {
                content_id,
                send_id,
                receive_id,
                content: proto::StudentInteractiveDataType::PeerToPeerCommunication {
                    content_id,
                    content,
                },
            }
        ), data_length).await
    }
}

#[derive(Debug)]
pub struct RefereeClientReader {
    stream: SplitStream<Framed<SerialStream, codec::RefereeCodec>>,
}

impl RefereeClientReader {
    pub async fn recv(&mut self) -> Option<Result<proto::Frame2, codec::RefereeCodecError>> {
        self.stream.next().await
    }
    pub async fn watch_radar(self) -> RefereeClientReaderWatch {
        RefereeClientReaderWatch::spawn_radar(self).await
    }
}

pub struct RefereeClientReaderWatch {
    join_handle: tokio::task::JoinHandle<()>,
    game_robot_hp: watch::Receiver<Option<(proto::TeamHP, proto::TeamHP)>>,
    game_robot_status: watch::Receiver<Option<proto::GameRobotStatus>>,
    game_status: watch::Receiver<Option<proto::GameStatus>>,
    radar_mark_data: watch::Receiver<Option<proto::RadarMarkData>>,
    event_data: watch::Receiver<Option<proto::EventData>>,
    stop_signal: mpsc::Sender<()>,
}

impl RefereeClientReaderWatch {
    async fn stop(self) -> Result<(), tokio::task::JoinError> {
        self.stop_signal.send(()).await.unwrap();
        self.join_handle.await
    }
    async fn spawn_radar(mut reader: RefereeClientReader) -> Self {
        let (game_robot_hp_tx, game_robot_hp) = watch::channel(None);
        let (game_robot_status_tx, game_robot_status) = watch::channel(None);
        let (game_status_tx, game_status) = watch::channel(None);
        let (radar_mark_data_tx, radar_mark_data) = watch::channel(None);
        let (event_data_tx, event_data) = watch::channel(None);
        let (stop_signal, mut stop_signal_rx) = mpsc::channel(1);
        let join_handle = tokio::spawn(async move {
            loop {
                let frame = select! {
                    Some(frame) = reader.recv() => frame,
                    _ = stop_signal_rx.recv() => { break; }
                    else => { break; }
                };
                match frame {
                    Ok(frame) => match frame.message {
                        proto::Message::GameRobotHP { red, blue } => { game_robot_hp_tx.send_replace(Some((red, blue))); }
                        proto::Message::GameRobotStatus(status) => { game_robot_status_tx.send_replace(Some(status)); }
                        proto::Message::GameStatus(status) => { game_status_tx.send_replace(Some(status)); }
                        proto::Message::RadarMarkData(data) => { radar_mark_data_tx.send_replace(Some(data)); }
                        proto::Message::EventData(data) => { event_data_tx.send_replace(Some(data)); }
                        proto::Message::DartRemainingTime(_) |
                        proto::Message::GameRobotPos { .. } |
                        proto::Message::RFIDStatus { .. } |
                        proto::Message::PowerRuneBuff { .. } |
                        proto::Message::PowerHeatData { .. } => {}
                        _ => {
                            debug!("Unhandled message: {:?}", frame.message);
                        }
                    },
                    Err(err) => {
                        error!("Error while receiving frame: {}", err);
                    }
                }
            }
        }.instrument(tracing::info_span!("watch_radar")));
        Self {
            join_handle,
            stop_signal,
            game_robot_hp,
            game_robot_status,
            game_status,
            radar_mark_data,
            event_data,
        }
    }
    pub async fn get_game_robot_hp(&mut self) -> (proto::TeamHP, proto::TeamHP) {
        self.game_robot_hp.wait_for(Option::is_some).await.unwrap().clone().unwrap()
    }
    pub async fn get_game_robot_status(&mut self) -> proto::GameRobotStatus {
        self.game_robot_status.wait_for(Option::is_some).await.unwrap().clone().unwrap()
    }
    pub async fn get_game_status(&mut self) -> proto::GameStatus {
        self.game_status.wait_for(Option::is_some).await.unwrap().clone().unwrap()
    }
    pub async fn get_radar_mark_data(&mut self) -> proto::RadarMarkData {
        self.radar_mark_data.wait_for(Option::is_some).await.unwrap().clone().unwrap()
    }
    pub async fn get_event_data(&mut self) -> proto::EventData {
        self.event_data.wait_for(Option::is_some).await.unwrap().clone().unwrap()
    }
}

mod codec {
    use std::io;

    use bytes::{Buf, BufMut, BytesMut};
    use deku::prelude::*;
    use tokio_util::codec::{Decoder, Encoder};
    use tracing::{debug, trace, warn};

    use crate::proto;

    #[derive(Debug)]
    pub struct RefereeCodec;

    #[derive(thiserror::Error, Debug)]
    pub enum RefereeCodecError {
        #[error("Internal decoder error")]
        Deku(#[from] DekuError),
        #[error("IO error")]
        Io(#[from] io::Error),
    }

    impl RefereeCodec {}

    impl Decoder for RefereeCodec {
        type Item = proto::Frame2;
        type Error = RefereeCodecError;

        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            if src.len() < 5 { return Ok(None); }
            if src[0] != 0xA5 {
                if let Some(sof_index) = src[1..].iter().position(|&b| b == 0xA5) {
                    let skipped = src.split_to(sof_index);
                    debug!("Fast-forwarding {} bytes for 0xA5: {}", sof_index, hex::encode(skipped));
                    if src.len() < 5 { return Ok(None); }
                } else {
                    return Ok(None);
                }
            }

            trace!("0xA5 is aligned");

            let calculated_crc = proto::crc::CRC_8.checksum(&src[0..4]);
            if calculated_crc != src[4] {
                debug!("Wrong CRC8: {} summed to {:x} instead of {:x}, fast-forwarding", hex::encode(&src[0..4]), calculated_crc,src[4]);
                src.advance(1);
                return Ok(None);
            }


            let data_length = u16::from_le_bytes([src[1], src[2]]) as usize;
            if data_length > 119 {
                debug!("Invalid data length: {}", data_length);
                src.advance(1);
                return Ok(None);
            }
            if src.len() < 5 + 2 + data_length + 2 {
                trace!("Not enough bytes for a complete frame ({} < {})", src.len(), 5 + 2 + data_length + 2);
                return Ok(None);
            }


            let calculated_crc = proto::crc::CRC_16.checksum(&src[0..7 + data_length]);
            if calculated_crc != u16::from_le_bytes([src[7 + data_length], src[7 + data_length + 1]]) {
                debug!("Wrong CRC16: {} summed to {:x} instead of {:x}, fast-forwarding",
                    hex::encode(&src[0..7 + data_length]), calculated_crc, u16::from_le_bytes([src[7 + data_length], src[7 + data_length + 1]]) );
                src.advance(1);
                return Ok(None);
            }

            let frame_bytes = src.split_to(7 + data_length + 2);
            let ((_, rest_byte_size), frame) = proto::Frame2::from_bytes((frame_bytes.as_ref(), 0))?;
            if rest_byte_size != 0 {
                warn!("Frame2::from_bytes returned rest_byte_size != 0");
            }
            trace!("Frame: {:?}", frame);
            Ok(Some(frame))
        }
    }

    impl Encoder<proto::Frame2> for RefereeCodec {
        type Error = RefereeCodecError;

        fn encode(&mut self, item: proto::Frame2, dst: &mut BytesMut) -> Result<(), Self::Error> {
            let len = item.data_length;
            let mut buf = item.to_bytes().map_err(RefereeCodecError::Deku)?;
            assert_eq!((len + 9) as usize, buf.len());
            let crc8 = proto::crc::CRC_8.checksum(&buf[..4]);
            buf[4] = crc8;
            let crc16 = proto::crc::CRC_16.checksum(&buf[..buf.len() - 2]).to_le_bytes();
            let len = buf.len();
            buf[len - 2] = crc16[0];
            buf[len - 1] = crc16[1];
            dst.reserve(buf.len());
            dst.put_slice(&buf);
            Ok(())
        }
    }
}
