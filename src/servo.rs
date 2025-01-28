use crate::{
    command::Command,
    hardware::{Instruction, ID},
    serial,
};
use serialport::SerialPort;

pub struct Servo {
    id: ID,
}

impl Servo {
    pub fn new(id: ID) -> Self {
        Self { id }
    }

    pub fn is_avaliable(&self, port: &mut Box<dyn SerialPort>) -> bool {
        let ping = Command::new(self.id, Instruction::Ping);

        let response = serial::packet_tx_rx(ping, port);

        matches!(response, Ok(Some(_)))
    }
}
