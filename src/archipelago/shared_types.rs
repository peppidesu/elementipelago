use serde::{Deserialize, Serialize};

pub(super) type PlayerID = usize;
pub(super) type TeamID = usize;
pub(super) type ItemID = isize;
pub(super) type LocationID = isize;

#[derive(Deserialize, Serialize, Debug)]
pub(super) struct APVersion {
    major: usize,
    minor: usize,
    build: usize,
    class: String,
}

impl Default for APVersion {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 6,
            build: 5,
            class: "Version".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(super) enum HintStatus {
    Unspecified = 0,
    NoPriority = 10,
    Avoid = 20,
    Priority = 30,
    Found = 40,
}

#[derive(Deserialize, Serialize)]
pub(super) enum ClientStatus {
    Unknown = 0,
    Connected = 5,
    Ready = 10,
    Playing = 20,
    Goal = 30,
}
