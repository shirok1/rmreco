use std::{io, thread};
use std::io::Read;
use std::sync::{Arc, atomic};
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::time::Duration;

use crossbeam_channel::{Receiver, unbounded};
use deku::prelude::*;
use serialport;
use serialport::SerialPort;
use tracing::{error, info, warn};

use crate::proto;

pub struct RefereeClient {
    port: Box<dyn SerialPort>,
    // read_thread: Option<thread::JoinHandle<io::Result<()>>>,
    background_reader: Option<BackgroundReader>,
}

pub struct BackgroundReader {
    thread: Option<thread::JoinHandle<io::Result<()>>>,
    // receiver: Receiver<Frame2>,
    should_stop: Arc<AtomicBool>,
}

impl Drop for BackgroundReader {
    fn drop(&mut self) {
        self.should_stop.store(true, atomic::Ordering::Relaxed);
        // self.thread.join().expect("TODO: panic message");
        if let Some(thread) = self.thread.take() {
            match thread.join() {
                Err(join_error) => error!("Error joining thread: {:?}", join_error),
                Ok(Err(thread_error)) => warn!("Error in reader thread: {:?}", thread_error),
                Ok(Ok(())) => info!("Reader thread joined successfully"),
            }
        }
    }
}

impl RefereeClient {
    pub fn try_new(path: &str) -> anyhow::Result<Self> {
        let port = serialport::new(path, 115200)
            .timeout(Duration::from_millis(1000)).open()?;
        Ok(Self { port, background_reader: None })
    }

    pub fn send_message_with_known_data_length(&mut self, message: proto::Message, data_length: u16) -> anyhow::Result<()> {
        // unsafe {
        //     static mut SEQ: u8 = 0;
        //     buf[3] = SEQ;
        //     SEQ = SEQ.wrapping_add(1);
        // }
        static ATOMIC_SEQ: AtomicU8 = AtomicU8::new(0);
        // buf[3] = ATOMIC_SEQ.fetch_add(1, atomic::Ordering::Relaxed);
        let frame = proto::Frame2 {
            data_length,
            seq: ATOMIC_SEQ.fetch_add(1, atomic::Ordering::Relaxed),
            crc8: 0,
            message,
            crc_frame_tail: 0,
        };
        let mut buf: Vec<u8> = frame.to_bytes()?;

        let crc8 = proto::crc::CRC_8.checksum(&buf[..4]);
        buf[4] = crc8;
        let crc16 = proto::crc::CRC_16.checksum(&buf[..buf.len() - 2]).to_le_bytes();
        // buf[buf.len() - 2..buf.len()] = crc_16.to_le_bytes();
        let len = buf.len();
        buf[len - 2] = crc16[0];
        buf[len - 1] = crc16[1];
        self.port.write_all(&buf)?;
        Ok(())
    }

    /// 由雷达站发送给裁判系统，用于告知己方操作手对方机器人的位置
    ///
    /// 不论雷达站属于哪一方，都以小地图左下角（红方补给区）为原点，双方对向为第一维，范围为 `(0.0..28.0, 0.0..15.0)`
    pub fn send_minimap_receipt(&mut self, target_robot_id: u16, target_position: (f32, f32)) -> anyhow::Result<()> {
        self.send_message_with_known_data_length(proto::Message::MinimapReceipt {
            target_robot_id,
            target_position,
        }, 10)
    }

    pub fn join_read_thread(&mut self) -> anyhow::Result<()> {
        // if let Some(read_thread) = self.background_reader.take() {
        //     read_thread.should_stop.store(true, atomic::Ordering::Relaxed);
        //     read_thread.thread.join().unwrap()?;
        // }
        self.background_reader = None;
        Ok(())
    }

    pub fn spawn_read_thread(&mut self) -> anyhow::Result<Receiver<proto::Frame2>> {
        let clone = self.port.try_clone()?;
        let (sender, receiver) = unbounded();

        let should_stop = Arc::new(AtomicBool::new(false));
        let should_stop_clone = should_stop.clone();
        let thread = thread::spawn(move || -> io::Result<()> {
            let mut buf_reader = io::BufReader::new(clone);
            while !should_stop_clone.load(atomic::Ordering::Relaxed) {
                let mut header_buf = [0u8; 5];
                let mut skip_count = 0;
                while !should_stop_clone.load(atomic::Ordering::Relaxed) {
                    // let mut buf = [0u8; 1];
                    if let Err(err) = buf_reader.read_exact(&mut header_buf[0..=0]) {
                        match err.kind() {
                            io::ErrorKind::UnexpectedEof => {
                                error!("Unexpected EOF, probably disconnected!");
                                Err(err)?;
                            }
                            _ => { Err(err)?; }
                        }
                    }
                    if header_buf[0] == 0xA5 { break; }
                    // if buf[0] == 0xA5 { break; }
                    skip_count += 1;
                }
                if should_stop_clone.load(atomic::Ordering::Relaxed) { break; }

                if skip_count > 0 {
                    warn!("Skipped {} bytes", skip_count);
                }

                // let mut header_buf = [0u8; 4];
                buf_reader.read_exact(&mut header_buf[1..=4])?;
                let data_length = u16::from_le_bytes([header_buf[1], header_buf[2]]) as usize;
                if data_length > 119 {
                    warn!("Invalid data length: {}", data_length);
                    continue;
                }
                let calculated_crc = proto::crc::CRC_8.checksum(&header_buf[0..4]);
                if calculated_crc != header_buf[4] { warn!("Wrong CRC8: {:?} summed to {} instead of {}", &header_buf[0..4], calculated_crc,header_buf[4]); }


                let mut buf = vec![0u8; 7 + data_length + 2];
                // buf[0] = 0xA5;
                // buf[1..5].copy_from_slice(&header_buf);
                buf[0..5].copy_from_slice(&header_buf);
                buf_reader.read_exact(&mut buf[5..7 + data_length + 2])?;

                match proto::Frame2::from_bytes((&buf, 0)) {
                    Ok(((_, rest_byte_size), frame)) => {
                        if rest_byte_size != 0 {
                            warn!("Invalid frame, rest byte size: {}", rest_byte_size);
                            continue;
                        }

                        let calculated_crc = Self::CRC_16.checksum(&buf[0..7 + data_length]);

                        if calculated_crc != u16::from_le_bytes([buf[7 + data_length], buf[7 + data_length + 1]]) {
                            // warn!("Wrong CRC16");
                            warn!("Wrong CRC16: {:?} summed to {} instead of {}", &buf[0..7 + data_length], calculated_crc, u16::from_le_bytes([buf[7 + data_length], buf[7 + data_length + 1]]) );
                        }

                        // match frame.message {
                        //     _ => todo!(),
                        // };

                        sender.send(frame).unwrap();
                        // info!("{:?}", frame.message);

                        // println!("{:?}", frame.message);
                    }
                    Err(err) => {
                        warn!("Invalid frame: {:?}, bytes: {:?}", err, &buf);
                        continue;
                    }
                }

                // let ((_, rest_byte_size), frame) = Frame2::from_bytes((&buf, 0))?;
                // assert_eq!(rest_byte_size, 0);
                //
                // println!("{:?}", frame.message);
            }
            info!("should_stop is true, stopping read thread");
            Ok(())
        });
        let receiver_clone = receiver;
        self.background_reader = Some(BackgroundReader { thread: Some(thread), /*receiver,*/ should_stop });
        Ok(receiver_clone)
    }
}
