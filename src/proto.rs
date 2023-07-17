use deku::prelude::*;
use serde::{Serialize, Deserialize};

pub mod crc {
    const CRC_8_ALGORITHM: crc::Algorithm<u8> = crc::Algorithm {
        init: 0xff,
        ..crc::CRC_8_MAXIM_DOW
    };
    const CRC_16_ALGORITHM: crc::Algorithm<u16> = crc::Algorithm {
        init: 0xffff,
        ..crc::CRC_16_KERMIT
    };
    pub(crate) const CRC_8: crc::Crc<u8> = crc::Crc::<u8>::new(&CRC_8_ALGORITHM);
    pub(crate) const CRC_16: crc::Crc<u16> = crc::Crc::<u16>::new(&CRC_16_ALGORITHM);
}

pub mod graphic;

#[cfg(test)]
mod tests;

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(magic = b"\xA5")]
pub struct Frame2 {
    // pub sof: u8,
    pub data_length: u16,
    pub seq: u8,
    pub crc8: u8,
    #[deku(ctx = "*data_length")]
    pub message: Message,
    pub crc_frame_tail: u16,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u16")]
#[deku(ctx = "frame_size: u16")]
pub enum Message {
    #[deku(id = "0x0001")]
    GameStatus {
        game_type: GameType,
        game_progress: GameProgress,
        stage_remain_time: u16,
        sync_timestamp: u64,
    },
    #[deku(id = "0x0002")]
    GameResult(Winner),
    #[deku(id = "0x0003")]
    GameRobotHP {
        red: TeamHP,
        blue: TeamHP,
    },
    // #[deku(id = "0x0004")]
    // DartStatus,
    // #[deku(id = "0x0005")]
    // ICRABuffDebuffZoneAndLurkStatus,
    #[deku(id = "0x0101")]
    EventData(EventData),
    #[deku(id = "0x0102")]
    SupplyProjectileAction {
        supplier: ProjectileSupplier,
        robot: ProjectileReloadingRobot,
        outlet_status: ProjectileOutletStatus,
        supplied_number: SuppliedProjectileNumber,
    },
    #[deku(id = "0x0104")]
    RefereeWarning(RefereeWarning),
    #[deku(id = "0x0105")]
    DartRemainingTime(u8),
    // #[deku(id = "0x0201")]
    // GameRobotStatus([u8; 27]),
    // #[deku(id = "0x0202")]
    // PowerHeatData([u8; 16]),
    // #[deku(id = "0x0203")]
    // GameRobotPos([u8; 16]),
    // #[deku(id = "0x0204")]
    // PowerRuneBuff(u8),
    #[deku(id = "0x0205")]
    AerialRobotEnergy(u8),
    // #[deku(id = "0x0206")]
    // RobotHurt(u8),
    // #[deku(id = "0x0207")]
    // ShootData([u8; 6]),
    // #[deku(id = "0x0208")]
    // BulletRemaining([u8; 2]),
    // #[deku(id = "0x0209")]
    // RFIDStatus([u8; 4]),
    // #[deku(id = "0x020A")]
    // DartClientCmd([u8; 12]), // TODO: Official typedef is different from description, verify

    #[deku(id = "0x0301")]
    StudentInteractiveData(
        #[deku(ctx = "frame_size")]
        StudentInteractiveData
    ),
    #[deku(id = "0x0302")]
    CustomControllerInteractiveData(
        #[deku(bytes_read = "frame_size - 9")]
        Vec<u8>
    ),
    #[deku(id = "0x0303")]
    MinimapTransmission {
        target_position: (i32, i32, i32),
        command_keyboard: u8,
        target_robot_id: u16,
    },
    #[deku(id = "0x0305")]
    MinimapReceipt {
        target_robot_id: u16,
        target_position: (f32, f32),
    },

    // #[deku(id = "0x0304")]
    // RemoteControl([u8; 12]),
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
#[deku(bits = "4")]
pub enum GameType {
    #[deku(id = "1")]
    RMUC,
    #[deku(id = "2")]
    RMUT,
    #[deku(id = "3")]
    RMUA,
    #[deku(id = "4")]
    RMUL3v3,
    #[deku(id = "5")]
    RMUL1v1,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
#[deku(bits = "4")]
pub enum GameProgress {
    #[deku(id = "0")]
    PreCompetitionStage,
    #[deku(id = "1")]
    SetupPeriod,
    #[deku(id = "2")]
    InitializationStage,
    #[deku(id = "3")]
    FiveSecondCountdown,
    #[deku(id = "4")]
    InCombat,
    #[deku(id = "5")]
    CalculatingCompetitionResults,
}


#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
pub enum Winner {
    #[deku(id = "0")]
    Draw,
    #[deku(id = "1")]
    Red,
    #[deku(id = "2")]
    Blue,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamHP {
    _1: u16,
    _2: u16,
    _3: u16,
    _4: u16,
    _5: u16,
    _7: u16,
    outpost: u16,
    base: u16,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// 己方补给站 1 号补血点占领状态
    #[deku(bits = "1")]
    pub restoration_zone_1_occupied: bool,

    /// 己方补给站 3 号补血点占领状态
    #[deku(bits = "1")]
    pub restoration_zone_2_occupied: bool,

    /// 己方补给站 3 号补血点占领状态
    #[deku(bits = "1")]
    pub restoration_zone_3_occupied: bool,

    /// 打击点占领状态
    #[deku(bits = "1")]
    pub attack_point_occupied: bool,

    /// 小能量机关激活状态
    #[deku(bits = "1")]
    pub small_power_rune_activated: bool,

    /// 大能量机关激活状态
    #[deku(bits = "1")]
    pub big_power_rune_activated: bool,

    /// 己方侧 R2/B2 环形高地占领状态
    #[deku(bits = "1")]
    pub r2b2_occupied: bool,

    /// 己方侧 R3/B3 梯形高地占领状态
    #[deku(bits = "1")]
    pub r3b3_occupied: bool,

    #[deku(bits = "1")]
    pub r4b4_occupied: bool,

    /// 己方基地护盾状态
    #[deku(bits = "1")]
    pub base_has_virtual_shield: bool,

    /// 己方前哨站状态
    #[deku(bits = "1")]
    #[deku(pad_bits_after = "21")]
    pub outpost_survives: bool,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
pub enum ProjectileSupplier {
    #[deku(id = "1")]
    _1,
    #[deku(id = "2")]
    _2,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
pub enum ProjectileReloadingRobot {
    #[deku(id = "0")]
    None,
    #[deku(id = "1")]
    Red1Hero,
    #[deku(id = "2")]
    Red2Engineer,
    #[deku(id = "3")]
    Red3,
    #[deku(id = "4")]
    Red4,
    #[deku(id = "5")]
    Red5,

    #[deku(id = "101")]
    Blue1Hero,
    #[deku(id = "102")]
    Blue2Engineer,
    #[deku(id = "103")]
    Blue3,
    #[deku(id = "104")]
    Blue4,
    #[deku(id = "105")]
    Blue5,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
pub enum ProjectileOutletStatus {
    #[deku(id = "0")]
    Closed,
    #[deku(id = "1")]
    Preparing,
    #[deku(id = "2")]
    Dropping,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
pub enum SuppliedProjectileNumber {
    #[deku(id = "50")]
    _50,
    #[deku(id = "100")]
    _100,
    #[deku(id = "150")]
    _150,
    #[deku(id = "200")]
    _200,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(type = "u8")]
pub enum RefereeWarning {
    #[deku(id = "1")]
    YellowCard { foul_robot_id: u8 },
    #[deku(id = "2")]
    RedCard { foul_robot_id: u8 },
    #[deku(id = "3")]
    Forfeiture(#[deku(pad_bytes_after = "1")] ()),
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deku(ctx = "frame_size: u16")]
pub struct StudentInteractiveData {
    #[deku(update = "self.get_content_id()")]
    content_id: u16,
    send_id: u16,
    receive_id: u16,
    #[deku(ctx = "*content_id, frame_size")]
    content: StudentInteractiveDataType,
}

impl StudentInteractiveData {
    fn get_content_id(&self) -> u16 {
        match &self.content {
            StudentInteractiveDataType::PeerToPeerCommunication { content_id, .. } => *content_id,
            any => any.deku_id().unwrap(),
        }
    }
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Serialize, Deserialize)]
// #[deku(type = "u16")]
#[deku(ctx = "content_id: u16, frame_size: u16", id = "content_id")]
pub enum StudentInteractiveDataType {
    #[deku(id_pat = "0x0200..=0x02FF")]
    PeerToPeerCommunication {
        #[deku(skip, default = "content_id")] // TODO: remove work-around
        content_id: u16,
        #[deku(bytes_read = "frame_size - 9 - 6")]
        content: Vec<u8>,
    },
    #[deku(id = "0x0100")]
    GraphicDelete {
        operate_type: graphic::GraphicDeleteOperation,
        layer: u8,
    },
    #[deku(id = "0x0101")]
    GraphicDraw1([graphic::GraphicData; 1]),
    #[deku(id = "0x0102")]
    GraphicDraw2([graphic::GraphicData; 2]),
    #[deku(id = "0x0103")]
    GraphicDraw5([graphic::GraphicData; 5]),
    #[deku(id = "0x0104")]
    GraphicDraw7([graphic::GraphicData; 7]),
    #[deku(id = "0x0110")]
    GraphicDrawCharacter((graphic::GraphicData, [u8; 30])),
}
