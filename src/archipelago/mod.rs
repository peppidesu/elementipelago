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

#[derive(Event)]
struct StartConnect;

#[derive(Resource)]
struct ArchipelagoClient {
    ws: Option<Mutex<WsClient>>,
}

#[derive(Resource, Debug)]
struct ArchipelagoState {
    connected: bool,
    address: String,
    slot: String,
    password: String,
    slotdata: Option<SlotData>,

    found_items: Vec<NetworkItem>,
    checked_locations: Vec<LocationID>,
}

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
            let msg = vec![APClientMessage::Connect {
                password: state.password.clone(),
                game: "Elementipelago".to_string(),
                name: state.slot.clone(),
                uuid: Uuid::new_v4(),
                version: APVersion::default(),
                items_handling: 0b111,
                tags: Vec::new(),
                slot_data: true,
            }];
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
            state.connected = true;

            println!("Logged in, my state is {:?}", state);
        }
        APServerMessage::ReceivedItems { index, items } => {
            todo!("Send message about received items")
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
                Err(websocket::WebSocketError::Other(wb)) => {
                    break;
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                    panic!()
                }
                Ok(OwnedMessage::Text(str)) => match from_str(&str) {
                    Ok(ldes) => {
                        let ldes: Vec<APServerMessage> = ldes;
                        for des in ldes {
                            handle_server_message(&mut state, &mut ws, des)
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
            .add_systems(FixedUpdate, (poll_websocket, send_websocket_msg))
            .add_systems(Startup, init_state)
            .add_observer(init_connecting);
    }
}
