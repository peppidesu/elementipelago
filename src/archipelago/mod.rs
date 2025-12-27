use std::sync::Mutex;

use bevy::{asset::uuid::Uuid, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use websocket::{
    ClientBuilder, OwnedMessage, WebSocketError::NoDataAvailable, stream::sync::NetworkStream,
    sync::Client,
};

use server_messages::{APServerMessage, SlotData};

use crate::archipelago::{
    client_messages::APClientMessage,
    server_messages::NetworkItem,
    shared_types::{APVersion, ItemID, LocationID},
};

mod client_messages;
mod server_messages;
mod shared_types;

type WsClient = Client<Box<dyn NetworkStream + Send>>;

#[derive(Resource)]
struct ArchipelagoClient {
    ws: Option<Mutex<WsClient>>,
}

#[derive(Resource)]
struct ArchipelagoState {
    connected: bool,
    address: String,
    slot: String,
    password: String,
    slotdata: Option<SlotData>,

    found_items: Vec<NetworkItem>,
    checked_locations: Vec<LocationID>,
}

fn init_connecting(client: &mut Res<ArchipelagoClient>, mut state: ResMut<ArchipelagoState>) {
    if let Ok(client) = ClientBuilder::new(&format!("wss://{}", state.address)) {
        match client.connect(None) {
            Ok(_) => todo!(),
            Err(e) => eprintln!("can't connect to websocket "),
        }
    } else {
        eprintln!("Can't parse url")
    }
}

fn poll_websocket(mut client: ResMut<ArchipelagoClient>, mut state: ResMut<ArchipelagoState>) {
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
                Err(NoDataAvailable) => break,
                Err(e) => {
                    eprintln!("{}", e.to_string());
                    panic!()
                }
                Ok(OwnedMessage::Text(str)) => {
                    if let Ok(des) = from_str(&str) {
                        let des: APServerMessage = des;
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
                                    continue;
                                }
                                let msg = APClientMessage::Connect {
                                    password: state.password.clone(),
                                    game: "Elementipelago".to_string(),
                                    name: state.slot.clone(),
                                    uuid: Uuid::new_v4(),
                                    version: APVersion::default(),
                                    items_handling: 0b111,
                                    tags: Vec::new(),
                                    slot_data: true,
                                };

                                ws.send_message(&OwnedMessage::Text(
                                    serde_json::to_string(&msg).unwrap(),
                                ))
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
                                state.connected = true;
                            }
                            APServerMessage::ReceivedItems { index, items } => {
                                println!("Send message about received items")
                            }
                            APServerMessage::LocationInfo { locations } => {
                                println!("New info about a location")
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
                            APServerMessage::PrintJSON(print_jsonmessage) => todo!(),
                            APServerMessage::DataPackage { data } => todo!(),
                            APServerMessage::Bounced {} => todo!(),
                            APServerMessage::InvalidPacket {
                                typ,
                                original_cmd,
                                text,
                            } => todo!(),
                            APServerMessage::Retrieved {} => todo!(),
                            APServerMessage::SetReply {} => todo!(),
                        }
                    } else {
                        println!("non malformed server messages, not {:#?}", str);
                    }
                }

                _ => todo!("websocket message type not handled"),
            }
        }
    }
    if close_ws {
        client.ws = None;
    }
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
        app.insert_resource(ArchipelagoClient { ws: None })
            .insert_resource(ArchipelagoState {
                connected: false,
                address: "".to_string(),
                slot: "".to_string(),
                password: "".to_string(),
                slotdata: None,
                found_items: vec![],
                checked_locations: vec![],
            })
            .add_systems(FixedUpdate, (poll_websocket, send_websocket_msg));
    }
}
