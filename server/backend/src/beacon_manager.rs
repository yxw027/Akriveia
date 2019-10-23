
// this "manager" exists for two main purposes, first the initial implementation will involve
// serial communcation with the beacons, which will be synchronous. Since Actix is a single threaded
// event loop this would not be a good time for our webserver, so instead the work will be shoved
// off to "SyncArbitors" which gives an api like normal actix, but gives each actor(or each beacon
// in this case) its own communcation thread. The second reason is to hopefully abstract this
// functionality a little bit, so that it will be a little bit easier to move to the wireless
// implementation.


use actix::prelude::*;
use actix_web::Result;
use crate::beacon_dummy::*;
use crate::beacon_serial::*;
use crate::beacon_udp::*;
use crate::data_processor::*;
use crate::db_utils;
use crate::models::beacon;
use crate::models::network_interface;
use serialport;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct BeaconManager {
    pub data_processor: Addr<DataProcessor>,
    pub diagnostic_data: common::DiagnosticData,
    pub emergency: bool,
    pub pinger: SpawnHandle,
    pub serial_connections: Vec<mpsc::Sender<BeaconCommand>>,
    pub udp_connections: Vec<Addr<BeaconUDP>>,
}
impl Actor for BeaconManager {
    type Context = Context<Self>;
}

const VENDOR_WHITELIST: &[u16] = &[0x2341, 0x10C4];
const USE_DUMMY_BEACONS: bool = true;
const USE_SERIAL_BEACONS: bool = false;
const USE_UDP_BEACONS: bool = true;
lazy_static! {
    static ref PING_INTERVAL: Duration = Duration::from_millis(10000);
    static ref EMERGENCY_PING_INTERVAL: Duration = Duration::from_millis(1000);
}

pub enum BMCommand {
    EndEmergency,
    GetEmergency,
    ScanBeacons,
    StartEmergency,

    // internal
    Ping,
    Reboot,
}

impl Message for BMCommand {
    type Result = Result<common::SystemCommandResponse, io::Error>;
}

#[derive(Clone, Copy)]
pub enum BeaconCommand {
    EndEmergency,
    GetEmergency,
    ScanBeacons,
    StartEmergency,
    Ping,
    Reboot,
}

impl Message for BeaconCommand {
    type Result = Result<(), ()>;
}

impl BeaconManager {
    pub fn new(dp: Addr<DataProcessor>) -> Addr<BeaconManager> {
        BeaconManager::create(move |context| {
            let mut manager = BeaconManager {
                data_processor: dp,
                diagnostic_data: common::DiagnosticData::new(),
                emergency: false, // TODO get from db!
                serial_connections: Vec::new(),
                udp_connections: Vec::new(),
                pinger: Default::default(),
            };
            manager.ping_self(context, *PING_INTERVAL);
            manager
        })
    }

    fn ping_self(&mut self, ctx: &mut Context<Self>, dur: Duration) {
        ctx.cancel_future(self.pinger);
        self.pinger = ctx.run_interval(dur, |_actor, context| {
            context.notify(BMCommand::Ping);
        });
    }

    fn find_beacons(&mut self, context: &mut Context<Self>) {
        if USE_DUMMY_BEACONS { self.find_beacons_dummy(context); }
        if USE_SERIAL_BEACONS { self.find_beacons_serial(context); }
        if USE_UDP_BEACONS { self.find_beacons_udp(context); }
    }

    fn find_beacons_udp(&mut self, context: &mut Context<Self>) {
        let fut = db_utils::default_connect()
            .and_then(|client| {
                network_interface::select_network_interfaces(client)
                    .map(|(_client, ifaces)| {
                        ifaces
                    })
            })
            .into_actor(self)
            .and_then(|ifaces, actor, context| {
                for iface in ifaces {
                    match iface.beacon_port {
                        Some(port) => {
                            actor.udp_connections.push(BeaconUDP::new(context.address(), iface.ip.clone(), port as u16));
                        },
                        None => {},
                    }
                }
                fut::result(Ok(()))
            })
            .map_err(|err, _, _| {
                println!("failed to create udp connection {}", err);
            });
        context.spawn(fut);
    }

    fn find_beacons_dummy(&mut self, context: &mut Context<Self>) {
        let beacons_fut = db_utils::default_connect()
            .and_then(|client| {
                beacon::select_beacons(client)
                    .map(|(_client, beacons)| {
                        beacons
                    })
            })
            .into_actor(self)
            .and_then(|beacons, actor, context| {
                for beacon in beacons {
                    let (send, receive): (mpsc::Sender<BeaconCommand>, mpsc::Receiver<BeaconCommand>) = mpsc::channel();
                    let beacon_manager = context.address().clone();
                    thread::spawn(move || {
                        dummy_beacon_thread(beacon, receive, beacon_manager);
                    });
                    actor.serial_connections.push(send);
                }
                fut::result(Ok(()))
            })
            .map_err(|err, _, _| {
                println!("failed to create dummy beacons {}", err);
            });
        context.spawn(beacons_fut);
    }

    fn find_beacons_serial(&mut self, context: &mut Context<Self>) {
        if let Ok(avail_ports) = serialport::available_ports() {
            for port in avail_ports {
                println!("\t{}", port.port_name);
                let name = Box::new(port.port_name);
                match port.port_type {
                    serialport::SerialPortType::UsbPort(info) => {
                        // only print out, and keep track of, arduino usbs
                        if VENDOR_WHITELIST.iter().any( |vid| vid == &info.vid ) {
                            println!("\t\tType: USB");
                            println!("\t\tVID:{:04x}", info.vid);
                            println!("\t\tPID:{:04x}", info.pid);
                            println!("\t\tSerial Number: {}", info.serial_number.as_ref().map_or("", String::as_str));
                            println!("\t\tManufacturer: {}", info.manufacturer.as_ref().map_or("", String::as_str));
                            println!("\t\tProduct: {}", info.product.as_ref().map_or("", String::as_str));

                            let (serial_send, serial_receive): (mpsc::Sender<BeaconCommand>, mpsc::Receiver<BeaconCommand>) = mpsc::channel();
                            let address = context.address().recipient();
                            let beacon_info = BeaconSerialConn {
                                port_name: (*name).clone(),
                                vid: info.vid,
                                pid: info.pid,
                                receive: serial_receive,
                                manager: address.clone(),

                            };
                            thread::spawn(move || {
                                serial_beacon_thread(beacon_info);
                            });
                            self.serial_connections.push(serial_send);
                        }
                    }
                    _ => {}
                }
            }
        } else {
            print!("Error listing serial ports");
        }
    }

    fn mass_send(&self, msg: BeaconCommand) {
        for connection in &self.serial_connections {
            connection.send(msg)
                .expect("failed to send message to serial connection");
        }

        for connection in &self.udp_connections {
            connection.do_send(msg);
        }
    }
}

impl Handler<BMCommand> for BeaconManager {
    type Result = Result<common::SystemCommandResponse, io::Error>;

    fn handle(&mut self, msg: BMCommand, context: &mut Context<Self>) -> Self::Result {
        match msg {
            BMCommand::GetEmergency => { },
            BMCommand::ScanBeacons => {
                println!("find beacons called!");
                self.find_beacons(context);
            },
            BMCommand::StartEmergency => {
                self.diagnostic_data = common::DiagnosticData::new();
                self.mass_send(BeaconCommand::StartEmergency);
                self.emergency = true;
                self.ping_self(context, *EMERGENCY_PING_INTERVAL);
            }
            BMCommand::EndEmergency => {
                self.mass_send(BeaconCommand::EndEmergency);
                self.emergency = false;
                self.data_processor.do_send(DPMessage::ResetData);
                self.ping_self(context, *PING_INTERVAL);
            },
            BMCommand::Ping => {
                self.mass_send(BeaconCommand::Ping);
            },
            BMCommand::Reboot => {
                self.mass_send(BeaconCommand::Reboot);
            },

        }
        Ok(common::SystemCommandResponse::new(self.emergency))
    }
}

pub struct GetDiagnosticData;
impl Message for GetDiagnosticData {
    type Result = Result<common::DiagnosticData, io::Error>;
}
impl Handler<GetDiagnosticData> for BeaconManager {
    type Result = Result<common::DiagnosticData, io::Error>;

    fn handle(&mut self, _msg: GetDiagnosticData, _context: &mut Context<Self>) -> Self::Result {
        let res = self.diagnostic_data.clone();
        self.diagnostic_data.tag_data = Vec::new();
        Ok(res)
    }
}

pub struct TagDataMessage {
    pub data: common::TagData,
}
impl Message for TagDataMessage {
    type Result = Result<u64, io::Error>;
}
impl Handler<TagDataMessage> for BeaconManager {
    type Result = Result<u64, io::Error>;

    fn handle(&mut self, msg: TagDataMessage, context: &mut Context<Self>) -> Self::Result {
        self.diagnostic_data.tag_data.push(msg.data.clone());
        self.data_processor.do_send(InLocationData(msg.data.clone()));

        let update_beacon = db_utils::default_connect()
            .and_then(move |client| {
                beacon::update_beacon_stamp_by_mac(client, msg.data.beacon_mac, msg.data.timestamp)
                    .map(|(_client, _beacons)| {
                    })
            })
            .into_actor(self)
            .map_err(|err, _, _| {
                println!("failed to create dummy beacons {}", err);
            });
        context.spawn(update_beacon);

        Ok(1)
    }
}
