extern crate serialport;

use std::sync::{Arc, Mutex};
use serialport::prelude::*;
use actix::prelude::*;
use serialport::*;
use std::io::{self, Write};
use std::time::Duration;
use std::thread;
use crate::beacon_manager::*;


#[allow(dead_code)] // remove this once vid/pid are actually used.
pub struct BeaconSerialConn {
    pub port_name: String,
    pub vid: u16,
    pub pid: u16,
    pub port: Option<Box<SerialPort>>,
}

impl Actor for BeaconSerialConn {
    type Context = SyncContext<Self>;
}

pub struct StartDataCollection;
impl Message for StartDataCollection {
    type Result = Result<u64>;
}

pub struct GetBeaconData;
impl Message for GetBeaconData {
    type Result = Result<Vec<common::TagData>>;
}

impl Handler<StartDataCollection> for BeaconSerialConn {
    type Result = Result<u64>;

    fn handle(&mut self, msg: StartDataCollection, context: &mut SyncContext<Self>) -> Self::Result {

        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(10);
        settings.baud_rate = 9600;
        match serialport::open_with_settings(&self.port_name, &settings) {
            Ok(mut opened_port) => {
                println!("clearing port");

                for i in 1.. {
                    thread::sleep(Duration::from_millis(300));
                    if let Ok(_) = opened_port.clear(ClearBuffer::All) {}

                    thread::sleep(Duration::from_millis(300));
                    println!("initiating communication, attempt {}", i);
                    if let Ok(_) = opened_port.write(b"start") {};

                    let mut serial_buffer: Vec<u8> = vec![0; 1000];
                    match opened_port.read(serial_buffer.as_mut_slice()) {
                        Ok(_) => {
                            let result = String::from_utf8_lossy(&serial_buffer);
                            if result == "ack" {
                                println!("successfully received ack from beacon");
                                break;
                            } else {
                                println!("failed to start beacon");
                            }

                        },
                        Err(_) => {
                            println!("serial communication failed on reading ack");
                        }
                    }
                }
                self.port = Some(opened_port);
            }
            Err(e) => {
                eprintln!("Failed to open arduino port \"{}\". Error: {}", self.port_name, e);
            }
        }

        Ok(1)
    }
}

impl Handler<GetBeaconData> for BeaconSerialConn {
    type Result = Result<Vec<common::TagData>>;

    fn handle(&mut self, msg: GetBeaconData, context: &mut SyncContext<Self>) -> Self::Result {

        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(10);
        settings.baud_rate = 9600;

        let mut tag_data: Vec<common::TagData> = Vec::new();
        match &self.port as Option<Box<SerialPort>> {
            Some(&mut opened_port) => {
                let mut serial_buffer: Vec<u8> = vec![0; 1000];
                println!("Receiving data on {} :", &self.port_name);
                loop {
                    println!("reading...");
                    thread::sleep(Duration::from_millis(100));
                    for _ in 1..3 {
                        match opened_port.read(serial_buffer.as_mut_slice()) {
                            Ok(t) => io::stdout().write_all(&serial_buffer[..t]).unwrap(),
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }

                    thread::sleep(Duration::from_millis(1000));

                    tag_data.push(common::TagData {
                        name: "hello".to_string(),
                        mac_address: "bleh bleh".to_string(),
                        distance: common::DataType::RSSI(55),
                    });
                }
            }
            None => {
                eprintln!("Failed to open arduino port \"{}\". ", self.port_name);
            }
        }

        println!("returning from get beacondata");
        Ok(tag_data)
    }
}
