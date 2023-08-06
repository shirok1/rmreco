use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Red,
    Blue,
}

impl Side {
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Side::Red),
            100 => Some(Side::Blue),
            _ => None,
        }
    }
    pub const fn opposite(&self) -> Self {
        match self {
            Side::Red => Side::Blue,
            Side::Blue => Side::Red,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RobotJob {
    Hero,
    Engineer,
    Infantry3,
    Infantry4,
    Infantry5,
    Sentry,
    Drone,
    Dart,
    Radar,
}

impl RobotJob {
    const LAND_ROBOT: [RobotJob; 6] = [
        RobotJob::Hero,
        RobotJob::Engineer,
        RobotJob::Infantry3,
        RobotJob::Infantry4,
        RobotJob::Infantry5,
        RobotJob::Sentry,
    ];
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(RobotJob::Hero),
            2 => Some(RobotJob::Engineer),
            3 => Some(RobotJob::Infantry3),
            4 => Some(RobotJob::Infantry4),
            5 => Some(RobotJob::Infantry5),
            6 => Some(RobotJob::Sentry),
            7 => Some(RobotJob::Drone),
            8 => Some(RobotJob::Dart),
            9 => Some(RobotJob::Radar),
            _ => None,
        }
    }
    pub const fn to_id(&self) -> u8 {
        match self {
            RobotJob::Hero => 1,
            RobotJob::Engineer => 2,
            RobotJob::Infantry3 => 3,
            RobotJob::Infantry4 => 4,
            RobotJob::Infantry5 => 5,
            RobotJob::Sentry => 6,
            RobotJob::Drone => 7,
            RobotJob::Dart => 8,
            RobotJob::Radar => 9,
        }
    }
    pub const fn from_id_with_side(id: u8) -> Option<(Side, Self)> {
        match id {
            0..=99 => Some((Side::Red, match RobotJob::from_id(id) {
                None => return None,
                Some(rj) => rj,
            })),
            100..=199 => Some((Side::Blue, match RobotJob::from_id(id - 100) {
                None => return None,
                Some(rj) => rj,
            })),
            _ => None,
        }
    }
    pub const fn convert_to_id((side, job): &(Side, Self)) -> u8 {
        match side {
            Side::Red => job.to_id(),
            Side::Blue => job.to_id() + 100,
        }
    }
}
