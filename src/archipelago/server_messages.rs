use super::shared_types::*;
use bevy::platform::collections::HashMap;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Deserialize_repr)]
#[repr(u8)]
pub(super) enum Permission {
    Disabled = 0b000,
    Enabled = 0b001,
    Goal = 0b010,
    Auto = 0b110,
    AutoEnabled = 0b111,
}

#[derive(Deserialize, Debug)]
pub(super) enum APConnectionError {
    InvalidSlot,
    InvalidGame,
    IncompatibleVersion,
    InvalidPassword,
    InvalidItemsHandling,
}

#[derive(Deserialize)]
pub(super) struct Permissions {
    release: Permission,
    collect: Permission,
    remaining: Permission,
}

#[derive(Deserialize)]
pub(super) struct NetworkPlayer {
    team: TeamID,
    slot: PlayerID,
    alias: String,
    name: String,
}

#[derive(Deserialize, Debug)]
pub(super) struct NetworkItem {
    pub(super) item: ItemID,
    pub(super) location: LocationID,
    pub(super) player: PlayerID,
    flags: u8,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct SlotData {
    element_amount: usize,
    filler_amount: usize,
    intermediate_amount: usize,
    graph_seed: usize,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[repr(u8)]
pub(super) enum NetworkSlot {
    Spectator {
        name: String,
        game: String,
    } = 0b00,
    Player {
        name: String,
        game: String,
    } = 0b01,
    Group {
        name: String,
        game: String,
        group_members: Vec<usize>,
    } = 0b10,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct DataPackageObject {
    pub(super) checksum: String,
    #[serde(default)]
    item_name_groups: HashMap<String, Vec<String>>,
    pub(super) item_name_to_id: HashMap<String, ItemID>,
    #[serde(default)]
    location_name_groups: HashMap<String, Vec<String>>,
    pub(super) location_name_to_id: HashMap<String, LocationID>,
}

#[derive(Deserialize, Debug)]
pub(super) struct DataPackageGames {
    pub(super) games: HashMap<String, DataPackageObject>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct JsonData {
    text: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type")]
pub(super) enum PrintJSONMessage {
    Join {
        data: Vec<JsonData>,
        team: TeamID,
        slot: PlayerID,
        tags: Vec<String>,
    },
    Tutorial {
        data: Vec<JsonData>,
    },
    #[serde(untagged)]
    Text {
        data: Vec<JsonData>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "cmd")]
pub(super) enum APServerMessage {
    RoomInfo {
        version: APVersion,
        generator_version: APVersion,
        tags: Vec<String>,
        password: bool,
        permissions: Permissions,
        hint_cost: isize,
        location_check_points: isize,
        games: Vec<String>,
        datapackage_checksums: HashMap<String, String>,
        seed_name: String,
        time: f64,
    },
    ConnectionRefused {
        errors: Vec<APConnectionError>,
    },
    Connected {
        team: TeamID,
        slot: PlayerID,
        players: Vec<NetworkPlayer>,
        missing_locations: Vec<LocationID>,
        checked_locations: Vec<LocationID>,
        slot_data: SlotData,
        slot_info: HashMap<String, NetworkSlot>,
        hint_points: isize,
    },
    ReceivedItems {
        index: isize,
        items: Vec<NetworkItem>,
    },
    LocationInfo {
        locations: Vec<NetworkItem>,
    },
    RoomUpdate {
        version: Option<APVersion>,
        generator_version: Option<APVersion>,
        tags: Option<Vec<String>>,
        password: Option<bool>,
        permissions: Option<Permissions>,
        hint_cost: Option<isize>,
        location_check_points: Option<isize>,
        games: Option<Vec<String>>,
        datapackage_checksums: Option<HashMap<String, String>>,
        seed_name: Option<String>,
        time: Option<f64>,
        team: Option<usize>,
        slot: Option<usize>,
        players: Option<Vec<NetworkPlayer>>,
        checked_locations: Option<Vec<isize>>,
        slot_data: Option<SlotData>,
        slot_info: Option<HashMap<usize, NetworkSlot>>,
        hint_points: Option<isize>,
    },
    PrintJSON(PrintJSONMessage),
    DataPackage {
        data: DataPackageGames,
    },
    Bounced {
        // IDK when this is used
    },
    InvalidPacket {
        #[serde(rename = "type")]
        typ: String,
        original_cmd: Option<String>,
        text: String,
    },
    Retrieved {
        // TODO: it's arbitrary data...
    },
    SetReply {
        // TODO: also arbitrary data
    },
}
