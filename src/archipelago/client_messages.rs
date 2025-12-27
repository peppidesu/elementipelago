use super::shared_types::*;
use bevy::asset::uuid::Uuid;
use serde::Serialize;

#[derive(Serialize)]
pub(super) enum APClientMessage {
    Connect {
        password: String,
        game: String,
        name: String,
        uuid: Uuid,
        version: APVersion,
        items_handling: u8, // Want 0b111
        tags: Vec<String>,
        slot_data: bool, // Want true
    },
    ConnectUpdata {
        items_handling: u8, // Want 0b111
        tags: Vec<String>,
    },
    Sync {
        // send to get RecievedItems refresh
    },
    LocationChecks {
        locations: Vec<LocationID>,
    },
    LocationScouts {
        locations: Vec<LocationID>,
        create_as_hint: isize,
    },
    CreateHints {
        locations: Vec<LocationID>,
        player: PlayerID,
        status: HintStatus,
    },
    UpdateHint {
        player: PlayerID,
        location: LocationID,
        status: HintStatus,
    },
    Statusupdate {
        status: ClientStatus,
    },
    Say {
        text: String,
    },
    GetDataPackage {
        games: Vec<String>,
    },
    Bounce {
        // TODO: arbitrary data I don't want to deal with rn
    },
    Get {
        // TODO: arbitrary data I don't want to deal with rn
    },
    Set {
        // TODO: arbitrary data I don't want to deal with rn
    },
    SetNotify {
        // TODO: arbitrary data I don't want to deal with rn
    },
}
