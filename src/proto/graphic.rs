use deku::prelude::*;
use serde::{Serialize, Deserialize};

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
pub enum GraphicDeleteOperation {
    #[deku(id = "0")]
    Nop,
    #[deku(id = "1")]
    DeleteOne,
    #[deku(id = "2")]
    DeleteAll,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicData {
    pub graphic_name: [u8; 3],
    pub operate_type: GraphicAddOperation,
    #[deku(bits = 3)]
    pub graphic_type: u8,
    #[deku(bits = 4)]
    pub layer: u8,
    pub color: GraphicColor,
    #[deku(ctx = "*graphic_type")]
    pub graphic_data: GraphicEnum,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
#[deku(bits = 3)]
pub enum GraphicAddOperation {
    #[deku(id = "0")]
    Nop,
    #[deku(id = "1")]
    Add,
    #[deku(id = "2")]
    Modify,
    #[deku(id = "3")]
    Delete,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(ctx = "graphic_type: u8", id = "graphic_type")]
pub enum GraphicEnum {
    #[deku(id = "0")]
    StraightLine(StraightLineRectangleData),
    #[deku(id = "1")]
    Rectangle(StraightLineRectangleData),
    #[deku(id = "2")]
    Circle(CircleData),
    #[deku(id = "3")]
    Ellipse(EllipseData),
    #[deku(id = "4")]
    Arc(ArcData),
    #[deku(id = "5")]
    FloatingNumber(FloatingNumberData),
    #[deku(id = "6")]
    Integer(IntegerData),
    #[deku(id = "7")]
    Character(CharacterData),
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StraightLineRectangleData {
    // start_angle: 9, end_angle: 9
    #[deku(pad_bits_before = "18", bits = 10)]
    pub width: u16,
    #[deku(bits = 11)]
    pub start_x: u16,
    #[deku(bits = 11)]
    pub start_y: u16,
    // radius: 10
    #[deku(pad_bits_before = "10", bits = 11)]
    pub end_x: u16,
    #[deku(bits = 11)]
    pub end_y: u16,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleData {
    // start_angle: 9, end_angle: 9
    #[deku(pad_bits_before = "18", bits = 10)]
    pub width: u16,
    #[deku(bits = 11)]
    pub x: u16,
    #[deku(bits = 11)]
    pub y: u16,
    #[deku(bits = 10, pad_bits_after = "22")]
    pub radius: u16,
    // end_x: 11, end_y: 11
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EllipseData {
    // start_angle: 9, end_angle: 9
    #[deku(pad_bits_before = "18", bits = 10)]
    pub width: u16,
    #[deku(bits = 11)]
    pub x: u16,
    #[deku(bits = 11)]
    pub y: u16,
    // radius: 10
    #[deku(pad_bits_before = "10", bits = 11)]
    pub half_x_length: u16,
    #[deku(bits = 11)]
    pub half_y_length: u16,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcData {
    #[deku(bits = 9)]
    pub start_angle: u16,
    #[deku(bits = 9)]
    pub end_angle: u16,
    #[deku(bits = 10)]
    pub width: u16,
    #[deku(bits = 11)]
    pub x: u16,
    #[deku(bits = 11)]
    pub y: u16,
    // radius: 10
    #[deku(pad_bits_before = "10", bits = 11)]
    pub half_x_length: u16,
    #[deku(bits = 11)]
    pub half_y_length: u16,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingNumberData {
    #[deku(bits = 9)]
    pub font_size: u16,
    #[deku(bits = 9)]
    pub decimal_digit: u16,
    #[deku(bits = 10)]
    pub width: u16,
    #[deku(bits = 11)]
    pub start_x: u16,
    #[deku(bits = 11)]
    pub start_y: u16,
    pub value: i32,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegerData {
    #[deku(bits = 9)]
    pub font_size: u16,
    // end_angle: 9,
    #[deku(pad_bits_before = "9", bits = 10)]
    pub width: u16,
    #[deku(bits = 11)]
    pub start_x: u16,
    #[deku(bits = 11)]
    pub start_y: u16,
    pub value: i32,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    #[deku(bits = 9)]
    pub font_size: u16,
    #[deku(bits = 9)]
    pub decimal_digit: u16,
    #[deku(bits = 10)]
    pub width: u16,
    #[deku(bits = 11)]
    pub x: u16,
    #[deku(bits = 11, pad_bits_after = "32")]
    pub y: u16,
    // radius:10, end_x: 11, end_y: 11
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
#[deku(bits = 4)]
pub enum GraphicColor {
    #[deku(id = "0")]
    RedAndBlue,
    #[deku(id = "1")]
    Yellow,
    #[deku(id = "2")]
    Green,
    #[deku(id = "3")]
    Orange,
    #[deku(id = "4")]
    PurplishRed,
    #[deku(id = "5")]
    Pink,
    #[deku(id = "6")]
    Cyan,
    #[deku(id = "7")]
    Black,
    #[deku(id = "8")]
    White,
}
