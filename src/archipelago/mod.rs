use std::{io::ErrorKind, sync::Mutex};

use bevy::{asset::uuid::Uuid, platform::collections::HashMap, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use websocket::{
    ClientBuilder, OwnedMessage, WebSocketError::NoDataAvailable, stream::sync::NetworkStream,
    sync::Client,
};

use server_messages::{APServerMessage, SlotData};

use crate::archipelago::{
    client_messages::APClientMessage,
    consts::ELEMENT_ID_OFFSET,
    server_messages::{DataPackageObject, NetworkItem},
    shared_types::{APVersion, ItemID, LocationID, PlayerID},
};

mod client_messages;
mod consts;
mod server_messages;
mod shared_types;

type WsClient = Client<Box<dyn NetworkStream + Send>>;

#[derive(Event)]
struct StartConnect;

#[derive(Resource)]
struct ArchipelagoClient {
    ws: Option<Mutex<WsClient>>,
}

#[derive(Debug)]
struct MyDataPackage {
    checksum: String,
    location_name_to_id: HashMap<String, LocationID>,
    location_id_to_name: HashMap<LocationID, String>,
    item_name_to_id: HashMap<String, ItemID>,
    item_id_to_name: HashMap<ItemID, String>,
}

#[derive(Resource, Debug)]
struct ArchipelagoState {
    connected: bool,
    address: String,
    slot: String,
    password: String,
    slotdata: Option<SlotData>,
    player_id: PlayerID,

    checked_locations: Vec<LocationID>,

    data_packages: HashMap<String, MyDataPackage>,
    games: HashMap<PlayerID, String>,
}

#[derive(Message, Debug)]
struct ReceivedItemMessage {
    item_name: String,
    related_location_name: String,
    graph_index_num: usize,
}

#[derive(Message, Default)]
struct GoSaveDataPackages;

fn init_connecting(
    _start: On<StartConnect>,
    mut apclient: ResMut<ArchipelagoClient>,
    mut state: ResMut<ArchipelagoState>,
) {
    if let Ok(mut client) = ClientBuilder::new(&format!("wss://{}", state.address)) {
        match client.connect(None) {
            Ok(client) => {
                client.set_nonblocking(true).unwrap();
                apclient.ws = Some(Mutex::new(client));
            }
            Err(TlsHandshakeFailure) => {
                let c = ClientBuilder::new(&format!("ws://{}", state.address))
                    .unwrap()
                    .connect(None)
                    .unwrap();
                c.set_nonblocking(true).unwrap();
                apclient.ws = Some(Mutex::new(c));
            }
            Err(e) => eprintln!("can't connect to websocket due to error {:?}", e),
        }
    } else {
        eprintln!("Can't parse url")
    }
}

fn handle_server_message(
    state: &mut ResMut<ArchipelagoState>,
    ws: &mut WsClient,
    des: APServerMessage,
    receive_writer: &mut MessageWriter<ReceivedItemMessage>,
    save_datapackages_writer: &mut MessageWriter<GoSaveDataPackages>,
) {
    match des {
        APServerMessage::RoomInfo {
            version,
            generator_version,
            tags,
            password,
            permissions,
            hint_cost,
            location_check_points,
            games,
            datapackage_checksums,
            seed_name,
            time,
        } => {
            if state.connected {
                return;
            }

            let mut to_fetch = Vec::new();
            for (game, checksum) in datapackage_checksums {
                println!("got datapackage checksum for {}: {}", game, checksum);

                if state.data_packages.contains_key(&game) {
                    if state.data_packages[&game].checksum != checksum {
                        to_fetch.push(game);
                    }
                } else {
                    to_fetch.push(game);
                }
            }
            let mut msg = vec![];
            if to_fetch.len() > 0 {
                msg.push(APClientMessage::GetDataPackage { games: to_fetch });
            }

            msg.push(APClientMessage::Connect {
                password: state.password.clone(),
                game: "Elementipelago".to_string(),
                name: state.slot.clone(),
                uuid: Uuid::new_v4(),
                version: APVersion::default(),
                items_handling: 0b111,
                tags: Vec::new(),
                slot_data: true,
            });
            println!(
                "I'm connecting, with json: {}",
                serde_json::to_string(&msg).unwrap()
            );

            ws.send_message(&OwnedMessage::Text(serde_json::to_string(&msg).unwrap()))
                .unwrap();
        }
        APServerMessage::ConnectionRefused { errors } => {
            state.connected = false;
            eprintln!("{:#?}", errors);
        }
        APServerMessage::Connected {
            team,
            slot,
            players,
            missing_locations,
            checked_locations,
            slot_data,
            slot_info,
            hint_points,
        } => {
            state.slotdata = Some(slot_data);
            state.checked_locations = checked_locations;
            state.player_id = slot;

            // this is the server
            state.games.insert(0, "Archipelago".to_string());
            for (slot, nwslot) in slot_info {
                println!("Game in slot {slot}: {:?}", nwslot);
                match nwslot {
                    server_messages::NetworkSlot::Spectator { name, game } => {
                        state
                            .games
                            .insert(slot.parse().expect("slot is integer"), game);
                    }
                    server_messages::NetworkSlot::Player { name, game } => {
                        state
                            .games
                            .insert(slot.parse().expect("slot is integer"), game);
                    }
                    server_messages::NetworkSlot::Group {
                        name,
                        game,
                        group_members,
                    } => todo!(),
                }
            }

            state.connected = true;

            println!("Logged in, my state is {:?}", state);
        }
        APServerMessage::ReceivedItems { index, items } => {
            receive_writer.write_batch(items.into_iter().map(|item| {
                let game = &state.games[&item.player];
                let data = &state.data_packages[game];
                let msg = ReceivedItemMessage {
                    item_name: state.data_packages["Elementipelago"].item_id_to_name[&item.item]
                        .clone(),
                    related_location_name: data.location_id_to_name[&item.location].clone(),
                    graph_index_num: item.item as usize - ELEMENT_ID_OFFSET,
                };
                // println!("Creating message: {msg:#?}");
                msg
            }));
        }
        APServerMessage::LocationInfo { locations } => {
            todo!("New info about a location")
        }
        APServerMessage::RoomUpdate {
            version,
            generator_version,
            tags,
            password,
            permissions,
            hint_cost,
            location_check_points,
            games,
            datapackage_checksums,
            seed_name,
            time,
            team,
            slot,
            players,
            checked_locations,
            slot_data,
            slot_info,
            hint_points,
        } => todo!(),
        APServerMessage::PrintJSON(print_jsonmessage) => {
            // TODO: print the jsonmessage for the user
            println!("Got server message {:#?}", print_jsonmessage);
        }
        APServerMessage::DataPackage { data } => {
            for (game, game_data) in data.games {
                state.data_packages.insert(
                    game,
                    MyDataPackage {
                        checksum: game_data.checksum,
                        location_id_to_name: game_data
                            .location_name_to_id
                            .iter()
                            .map(|(key, &value)| (value, key.clone()))
                            .collect(),
                        location_name_to_id: game_data.location_name_to_id,
                        item_id_to_name: game_data
                            .item_name_to_id
                            .iter()
                            .map(|(key, &value)| (value, key.clone()))
                            .collect(),
                        item_name_to_id: game_data.item_name_to_id,
                    },
                );
            }
            save_datapackages_writer.write_default();
        }
        APServerMessage::Bounced {} => todo!(),
        APServerMessage::InvalidPacket {
            typ,
            original_cmd,
            text,
        } => todo!(),
        APServerMessage::Retrieved {} => todo!(),
        APServerMessage::SetReply {} => todo!(),
    }
}

fn poll_websocket(
    mut client: ResMut<ArchipelagoClient>,
    mut state: ResMut<ArchipelagoState>,
    mut mw: MessageWriter<ReceivedItemMessage>,
    mut save_datapackages_writer: MessageWriter<GoSaveDataPackages>,
) {
    let mut close_ws = false;
    if let Some(mws) = &client.ws {
        let mut ws = mws.lock().expect("Mutex is poisoned");
        loop {
            match ws.recv_message() {
                // TODO: handle the errors better
                Ok(OwnedMessage::Ping(p)) => ws.send_message(&OwnedMessage::Pong(p)).unwrap(),
                Ok(OwnedMessage::Close(data)) => {
                    close_ws = true;
                    break;
                }
                Err(websocket::WebSocketError::IoError(wb)) => {
                    if wb.kind() == ErrorKind::WouldBlock {
                        break;
                    }
                    eprintln!("{:?}", wb);
                    panic!()
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                    panic!()
                }
                Ok(OwnedMessage::Text(str)) => match from_str(&str) {
                    Ok(ldes) => {
                        let ldes: Vec<APServerMessage> = ldes;
                        for des in ldes {
                            handle_server_message(
                                &mut state,
                                &mut ws,
                                des,
                                &mut mw,
                                &mut save_datapackages_writer,
                            )
                        }
                    }
                    Err(e) => {
                        println!("Can't decode: {}, got err {:?}", str, e);
                        panic!();
                    }
                },

                _ => todo!("websocket message type not handled"),
            }
        }
    }
    if close_ws {
        client.ws = None;
    }
}

fn init_state(mut commands: Commands, mut state: ResMut<ArchipelagoState>) {
    state.address = "localhost:38281".to_string();
    state.slot = "Player1".to_string();

    commands.trigger(StartConnect);
}

// run when an item is merged or something (or on a timer with messages)
fn send_websocket_msg(client: Res<ArchipelagoClient>, mut state: ResMut<ArchipelagoState>) {
    if !state.connected {
        return;
    }
}

pub struct ArchipelagoPlugin;

impl Plugin for ArchipelagoPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ReceivedItemMessage>()
            .add_message::<GoSaveDataPackages>()
            .insert_resource(ArchipelagoClient { ws: None })
            .insert_resource(ArchipelagoState {
                connected: false,
                address: "".to_string(),
                slot: "".to_string(),
                password: "".to_string(),
                slotdata: None,
                found_items: vec![],
                checked_locations: vec![],
                data_packages: HashMap::new(),
                games: HashMap::new(),
                player_id: 0,
            })
            .add_systems(FixedUpdate, (poll_websocket, send_websocket_msg))
            .add_systems(Startup, init_state)
            .add_observer(init_connecting);
    }
}
