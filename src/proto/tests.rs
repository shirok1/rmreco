use deku::bitvec::{BitView, Msb0};
use super::*;

#[test]
fn graphic_data_size() {
    let mut data = [0u8; 15];
    for i in 0b000..=0b111 {
        data[3] = i * 0b0000_0100;
        let ((_, rest_byte_size), _) = GraphicData::from_bytes((&data[..], 0)).unwrap();
        assert_eq!(rest_byte_size, 0);
    }
}

#[test]
fn referee_warning_size() {
    let mut data = [0u8; 2];
    for i in 0b01..=0b11 {
        data[0] = i;
        let ((_, rest_byte_size), _) = RefereeWarning::from_bytes((&data[..], 0)).unwrap();
        assert_eq!(rest_byte_size, 0);
    }
}

#[test]
fn student_interactive_data_size() {
    let mut data = Vec::new();
    for custom_size in 1..=113 {
        data.resize((2 + 6 + custom_size) as usize, 0);
        const CONTENT_ID: u16 = 0x0200;
        [data[0], data[1]] = 0x0301_u16.to_le_bytes();
        [data[2], data[3]] = CONTENT_ID.to_le_bytes();
        [data[4], data[5]] = 0x1234_u16.to_le_bytes();
        [data[6], data[7]] = 0x5678_u16.to_le_bytes();
        let (rest_bits, data) = Message::read(
            data.view_bits::<Msb0>(), 9 + 6 + custom_size).unwrap();
        assert_eq!(rest_bits.len(), 0);
        assert!(matches!(data,
            Message::StudentInteractiveData(
                StudentInteractiveData {
                    content_id: CONTENT_ID,
                    content: StudentInteractiveDataType::PeerToPeerCommunication { content_id: CONTENT_ID, .. }, ..
                })));
    }
    for custom_size in 1..=113 {
        data.resize((custom_size) as usize, 0);
        const CONTENT_ID: u16 = 0x0200;
        let (rest_bits, data) = StudentInteractiveDataType::read(
            data.view_bits::<Msb0>(), (CONTENT_ID, 9 + 6 + custom_size)).unwrap();
        assert_eq!(rest_bits.len(), 0);
        assert!(matches!(data, StudentInteractiveDataType::PeerToPeerCommunication { content_id: CONTENT_ID, .. }));
    }
}

#[test]
fn custom_controller_interactive_size() {
    let mut data = Vec::new();
    for custom_size in 1..=30 {
        data.resize((2 + custom_size) as usize, 0);
        [data[0], data[1]] = 0x0302_u16.to_le_bytes();
        let (rest_bits, _) = Message::read(data.view_bits::<Msb0>(), 9 + custom_size).unwrap();
        assert_eq!(rest_bits.len(), 0);
    }
}