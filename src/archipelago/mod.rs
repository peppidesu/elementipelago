use std::fs::read_to_string;
use std::time::Duration;

use bevy::asset::uuid::Uuid;
use bevy::platform::collections::HashSet;
use bevy::{platform::collections::HashMap, prelude::*};
use crossbeam_channel::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use url::Url;
use yawc::frame::{FrameView, OpCode};
use yawc::{CompressionLevel, Options, WebSocket};

use server_messages::{APServerMessage, SlotData};

use crate::archipelago::datapackage::{DataPackageSave, save_datapackage};
use crate::graph::Status;
use crate::{
    archipelago::{
        client_messages::APClientMessage,
        consts::ELEMENT_ID_OFFSET,
        shared_types::{APVersion, ItemID, LocationID, PlayerID},
    },
    game::RecipeGraph,
    graph,
};

mod client_messages;
mod consts;
mod datapackage;
mod server_messages;
mod shared_types;

#[derive(Event)]
pub struct StartConnect;

/// Commands sent from Bevy -> WS thread.
#[derive(Debug)]
enum WsCommand {
    /// Connect to the given URL (or host:port). The WS thread will try `wss://` first, then `ws://`.
    Connect { address: String },
    /// Send a text message (JSON).
    SendText(String),
    /// Close/shutdown the WS thread.
    Shutdown,
}

/// Events sent from WS thread -> Bevy.
#[derive(Debug)]
enum WsEvent {
    Connected { url: String },
    ConnectionError { error: String },
    Disconnected { reason: String },
    TextMessage(String),
}

#[derive(Resource)]
struct ArchipelagoClient {
    cmd_tx: Option<Sender<WsCommand>>,
    evt_rx: Option<Receiver<WsEvent>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct MyDataPackage {
    checksum: String,
    location_name_to_id: HashMap<String, LocationID>,
    location_id_to_name: HashMap<LocationID, String>,
    item_name_to_id: HashMap<String, ItemID>,
    item_id_to_name: HashMap<ItemID, String>,
}

#[derive(Resource, Debug)]
pub struct ArchipelagoState {
    connected: bool,
    pub address: String,
    pub slot: String,
    pub password: String,
    slotdata: Option<SlotData>,
    player_id: PlayerID,

    checked_locations: HashSet<LocationID>,

    data_packages: HashMap<String, MyDataPackage>,
    games: HashMap<PlayerID, String>,
    hint_points: isize,
}

fn init_datapackages() -> HashMap<String, MyDataPackage> {
    let dir = dirs::cache_dir().unwrap();
    let datapackage_dir = dir.join("elementipelago").join("datapackages");

    let mut data_packages = HashMap::new();

    let Ok(dir) = std::fs::read_dir(datapackage_dir) else {
        return data_packages;
    };

    for file in dir {
        let Ok(file) = file else {
            continue;
        };

        let Ok(content) = read_to_string(file.path()) else {
            continue;
        };

        let Ok(dps) = serde_json::from_str::<DataPackageSave>(&content) else {
            continue;
        };

        let game = dps.game;
        let datapackage = dps.datapackage;

        data_packages.insert(game, datapackage);
    }

    return data_packages;
}

impl Default for ArchipelagoState {
    fn default() -> Self {
        Self {
            connected: false,
            address: "".to_string(),
            slot: "".to_string(),
            password: "".to_string(),
            slotdata: None,
            player_id: 0,
            checked_locations: Default::default(),
            data_packages: init_datapackages(),
            games: Default::default(),
            hint_points: 0,
        }
    }
}

#[derive(Message, Debug, Default)]
pub struct ConnectedMessage;

#[derive(Message, Debug)]
pub struct SendItemMessage {
    pub element: graph::Element,
}

#[derive(Message, Debug)]
pub enum ConnectionErrorMessage {
    UriParseError,
    ConnectionFailed(String),
}

#[derive(Message, Debug)]
pub struct ReceivedItemMessage {
    pub element: graph::Element,
}

fn normalize_urls(input: &str) -> Vec<Url> {
    let wss = format!("wss://{input}");
    let ws = format!("ws://{input}");

    let mut out = Vec::new();
    if let Ok(u) = Url::parse(&wss) {
        out.push(u);
    }
    if let Ok(u) = Url::parse(&ws) {
        out.push(u);
    }
    out
}

fn spawn_ws_thread() -> (Sender<WsCommand>, Receiver<WsEvent>) {
    let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<WsCommand>();
    let (evt_tx, evt_rx) = crossbeam_channel::unbounded::<WsEvent>();

    std::thread::Builder::new()
        .name("archipelago_ws".to_string())
        .spawn(move || {
            // WS thread owns a tokio runtime.
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .worker_threads(1)
                .thread_name("archipelago_ws_tokio")
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = evt_tx.send(WsEvent::ConnectionError {
                        error: format!("failed to create tokio runtime: {e:?}"),
                    });
                    return;
                }
            };

            rt.block_on(async move {
                ws_thread_main(cmd_rx, evt_tx).await;
            });
        })
        .expect("failed to spawn archipelago_ws thread");

    (cmd_tx, evt_rx)
}

async fn ws_thread_main(cmd_rx: Receiver<WsCommand>, evt_tx: Sender<WsEvent>) {
    let mut ws: Option<WebSocket> = None;
    let mut connected_url: Option<String> = None;

    // Compression options: enable permessage-deflate negotiation with a reasonable level.
    // (Negotiation happens during handshake when compression is set.)
    let options = Options::default()
        .with_compression_level(CompressionLevel::new(6))
        .with_utf8()
        .with_no_delay();

    loop {
        // Drain commands quickly.
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                WsCommand::Connect { address } => {
                    // Drop any existing socket.
                    ws = None;
                    connected_url = None;

                    let candidates = normalize_urls(&address);

                    if candidates.is_empty() {
                        let _ = evt_tx.send(WsEvent::ConnectionError {
                            error: format!("could not parse address into ws/wss URL: {address}"),
                        });
                        continue;
                    }

                    let mut last_err: Option<String> = None;

                    for url in candidates {
                        let url_s = url.to_string();

                        // Connect using yawc builder so we can apply Options (compression etc).
                        let attempt = WebSocket::connect(url).with_options(options.clone()).await;

                        match attempt {
                            Ok(sock) => {
                                connected_url = Some(url_s.clone());
                                ws = Some(sock);
                                let _ = evt_tx.send(WsEvent::Connected { url: url_s });
                                last_err = None;
                                break;
                            }
                            Err(e) => {
                                last_err = Some(format!("{e:?}"));
                                continue;
                            }
                        }
                    }

                    if let Some(err) = last_err {
                        let _ = evt_tx.send(WsEvent::ConnectionError {
                            error: format!("failed to connect (after fallback attempts): {err}"),
                        });
                    }
                }
                WsCommand::SendText(text) => {
                    if let Some(sock) = ws.as_mut() {
                        // yawc expects frames; for application messages we send Text.
                        if let Err(e) = sock.send(FrameView::text(text).into()).await {
                            let _ = evt_tx.send(WsEvent::Disconnected {
                                reason: format!("send error: {e:?}"),
                            });
                            ws = None;
                            connected_url = None;
                        }
                    }
                }
                WsCommand::Shutdown => {
                    if let Some(sock) = ws.as_mut() {
                        let _ = sock
                            .send(FrameView::close(yawc::close::CloseCode::Normal, b"bye").into())
                            .await;
                    }
                    return;
                }
            }
        }

        // Read from socket if connected.
        if let Some(sock) = ws.as_mut() {
            // Use a small timeout so we can keep checking cmd_rx without async bridging.
            match tokio::time::timeout(Duration::from_millis(10), sock.next()).await {
                Ok(Some(frame)) => {
                    // `frame` is a full Frame; convert to a view and handle opcodes.
                    let view: yawc::frame::FrameView = frame.into();
                    match view.opcode {
                        OpCode::Text => {
                            // Options::with_utf8 ensures text is valid utf8, but payload is bytes.
                            match std::str::from_utf8(&view.payload) {
                                Ok(s) => {
                                    let _ = evt_tx.send(WsEvent::TextMessage(s.to_string()));
                                }
                                Err(e) => {
                                    let _ = evt_tx.send(WsEvent::Disconnected {
                                        reason: format!("invalid utf8 from server: {e:?}"),
                                    });
                                    ws = None;
                                    connected_url = None;
                                }
                            }
                        }
                        OpCode::Close => {
                            let _ = evt_tx.send(WsEvent::Disconnected {
                                reason: "server closed connection".to_string(),
                            });
                            ws = None;
                            connected_url = None;
                        }
                        // yawc auto-responds to Ping with Pong (client behavior),
                        // but still yields the frame; we can ignore.
                        _ => {}
                    }
                }
                Ok(None) => {
                    // Stream ended.
                    let _ = evt_tx.send(WsEvent::Disconnected {
                        reason: "connection ended".to_string(),
                    });
                    ws = None;
                    connected_url = None;
                }
                Err(_) => {
                    // timeout: just loop again
                }
            }
        } else {
            // Not connected: back off a bit to avoid busy looping.
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

// -------------------------
// Bevy systems
// -------------------------

fn init_connecting(
    _start: On<StartConnect>,
    mut apclient: ResMut<ArchipelagoClient>,
    mut state: ResMut<ArchipelagoState>,
    mut error_writer: MessageWriter<ConnectionErrorMessage>,
) {
    // Ensure worker exists.
    if apclient.cmd_tx.is_none() || apclient.evt_rx.is_none() {
        let (cmd_tx, evt_rx) = spawn_ws_thread();
        apclient.cmd_tx = Some(cmd_tx);
        apclient.evt_rx = Some(evt_rx);
    }

    // Ask the worker to connect (it will do wss->ws fallback automatically).
    let Some(cmd_tx) = apclient.cmd_tx.as_ref() else {
        error_writer.write(ConnectionErrorMessage::ConnectionFailed(
            "missing command sender".to_string(),
        ));
        return;
    };

    if state.address.trim().is_empty() {
        error_writer.write(ConnectionErrorMessage::UriParseError);
        return;
    }

    // Trigger connect
    let _ = cmd_tx.send(WsCommand::Connect {
        address: state.address.clone(),
    });
}

fn poll_websocket(
    mut commands: Commands,
    apclient: Res<ArchipelagoClient>,
    mut state: ResMut<ArchipelagoState>,
    mut connected_writer: MessageWriter<ConnectedMessage>,
    mut error_writer: MessageWriter<ConnectionErrorMessage>,
    mut receive_writer: MessageWriter<ReceivedItemMessage>,
    mut graph: ResMut<RecipeGraph>,
) {
    let Some(evt_rx) = apclient.evt_rx.as_ref() else {
        return;
    };

    // Drain all available events each tick.
    while let Ok(evt) = evt_rx.try_recv() {
        match evt {
            WsEvent::Connected { url: _ } => {}
            WsEvent::ConnectionError { error } => {
                state.connected = false;
                error_writer.write(ConnectionErrorMessage::ConnectionFailed(error));
            }
            WsEvent::Disconnected { reason: _ } => {
                state.connected = false;
            }
            WsEvent::TextMessage(text) => match from_str::<Vec<APServerMessage>>(&text) {
                Ok(des) => {
                    for msg in des {
                        handle_ap_message(
                            &mut commands,
                            &mut state,
                            msg,
                            &mut receive_writer,
                            &mut connected_writer,
                            &apclient,
                            &mut graph,
                        );
                    }
                }
                Err(e) => {
                    warn!("Can't decode AP message: {e:?}\nraw: {text}");
                }
            },
        }
    }
}

/// Put your existing APServerMessage handling here.
/// This stub keeps the file self-contained; replace body with your original match tree.
fn handle_ap_message(
    commands: &mut Commands,
    state: &mut ResMut<ArchipelagoState>,
    des: APServerMessage,
    receive_writer: &mut MessageWriter<ReceivedItemMessage>,
    connected_writer: &mut MessageWriter<ConnectedMessage>,
    apclient: &ArchipelagoClient,
    graph: &mut ResMut<RecipeGraph>,
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
                        println!(
                            "I have package checksum {} for game {}, which is different than {}",
                            state.data_packages[&game].checksum, game, checksum
                        );
                        to_fetch.push(game);
                    }
                } else {
                    to_fetch.push(game);
                }
            }
            let mut msg = vec![];
            if !to_fetch.is_empty() {
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

            if let Some(sender) = &apclient.cmd_tx {
                sender
                    .send(WsCommand::SendText(serde_json::to_string(&msg).unwrap()))
                    .expect("ap cmd_tx channel closed")
            };
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
            state.checked_locations = checked_locations.into_iter().collect();
            state.player_id = slot;
            state.hint_points = hint_points;

            graph.0 = Some(
                (state.slotdata.as_ref())
                    .expect("Just put value in slot_data")
                    .generate_graph(),
            );

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
            connected_writer.write_default();
        }
        APServerMessage::ReceivedItems { index, items } => {
            receive_writer.write_batch(items.into_iter().filter_map(|item| {
                let game = &state.games[&item.player];
                let data = &state.data_packages[game];

                if item.item >= 100 {
                    Some(ReceivedItemMessage {
                        element: (item.item as u64 - ELEMENT_ID_OFFSET + 1, Status::INPUT),
                    })
                } else {
                    println!("Skipping item: {item:#?} since it's not handled yet");
                    None
                }
            }));
        }
        APServerMessage::DataPackage { data } => {
            for (game, game_data) in data.games {
                let mdp = MyDataPackage {
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
                };
                let _ = save_datapackage(&game, &mdp);
                state.data_packages.insert(game, mdp);
            }
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
        } => {
            if let Some(version) = version {
                eprintln!("Unhandled room update: version, {version:?}")
            }
            if let Some(generator_version) = generator_version {
                eprintln!("Unhandled room update: generator_version, {generator_version:?}")
            }
            if let Some(tags) = tags {
                eprintln!("Unhandled room update: tags, {tags:?}")
            }
            if let Some(password) = password {
                eprintln!("Unhandled room update: password, {password:?}")
            }
            if let Some(permissions) = permissions {
                eprintln!("Unhandled room update: permissions, {permissions:?}")
            }
            if let Some(hint_cost) = hint_cost {
                eprintln!("Unhandled room update: hint_cost, {hint_cost:?}")
            }
            if let Some(location_check_points) = location_check_points {
                eprintln!("Unhandled room update: location_check_points, {location_check_points:?}")
            }
            if let Some(games) = games {
                eprintln!("Unhandled room update: games, {games:?}")
            }
            if let Some(datapackage_checksums) = datapackage_checksums {
                eprintln!("Unhandled room update: datapackage_checksums, {datapackage_checksums:?}")
            }
            if let Some(seed_name) = seed_name {
                eprintln!("Unhandled room update: seed_name, {seed_name:?}")
            }
            if let Some(time) = time {
                eprintln!("Unhandled room update: time, {time:?}")
            }
            if let Some(team) = team {
                eprintln!("Unhandled room update: team, {team:?}")
            }
            if let Some(slot) = slot {
                eprintln!("Unhandled room update: slot, {slot:?}")
            }
            if let Some(players) = players {
                eprintln!("Unhandled room update: players, {players:?}")
            }
            if let Some(checked) = checked_locations {
                state.checked_locations.extend(checked.into_iter());
            }
            if let Some(slot_data) = slot_data {
                eprintln!("Unhandled room update: slot_data, {slot_data:?}")
            }
            if let Some(slot_info) = slot_info {
                eprintln!("Unhandled room update: slot_info, {slot_info:?}")
            }
            if let Some(hint_points) = hint_points {
                state.hint_points = hint_points;
            }
        }
        message => {
            eprintln!(
                "Got message {:#?} but don't know what to do with it yet",
                message
            )
        }
    }
}

fn send_websocket_msg(
    mut read_send_item: MessageReader<SendItemMessage>,
    apclient: Res<ArchipelagoClient>,
    state: Res<ArchipelagoState>,
) {
    if !state.connected {
        return;
    }
    let Some(cmd_tx) = apclient.cmd_tx.as_ref() else {
        return;
    };
    read_send_item.read().for_each(|msg| {
        if msg.element.1 == Status::OUTPUT {
            if state.checked_locations.contains(&(msg.element.0 as isize)) {
                println!("Location already checked: {:?}", state.checked_locations);
                return;
            }
            cmd_tx
                .send(WsCommand::SendText(
                    serde_json::to_string(&vec![APClientMessage::LocationChecks {
                        locations: vec![msg.element.0 as isize],
                    }])
                    .expect("can't make json from client message"),
                ))
                .expect("can't send message to websocket queue");
        }
    });
}

pub struct ArchipelagoPlugin;

impl Plugin for ArchipelagoPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ReceivedItemMessage>()
            .add_message::<ConnectedMessage>()
            .add_message::<SendItemMessage>()
            .add_message::<ConnectionErrorMessage>()
            .insert_resource(ArchipelagoClient {
                cmd_tx: None,
                evt_rx: None,
            })
            .insert_resource(ArchipelagoState::default())
            .add_systems(FixedUpdate, (poll_websocket, send_websocket_msg))
            .add_observer(init_connecting);
    }
}
