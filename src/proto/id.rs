use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WithSide<T> {
    Red(T),
    Blue(T),
}

impl<T> WithSide<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> WithSide<U> {
        match self {
            WithSide::Red(t) => WithSide::Red(f(t)),
            WithSide::Blue(t) => WithSide::Blue(f(t)),
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
}

impl WithSide<RobotJob> {
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            1..=100 => Some(WithSide::Red(match RobotJob::from_id(id) {
                None => return None,
                Some(rj) => rj,
            })),
            101..=200 => Some(WithSide::Blue(match RobotJob::from_id(id - 100) {
                None => return None,
                Some(rj) => rj,
            })),
            _ => None,
        }
    }
    pub const fn to_id(&self) -> u8 {
        match self {
            WithSide::Red(job) => job.to_id(),
            WithSide::Blue(job) => job.to_id() + 100,
        }
    }
}