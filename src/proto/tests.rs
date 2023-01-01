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
