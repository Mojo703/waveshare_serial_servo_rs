use std::io;

use crate::{
    command::Command,
    hardware::{Address, DriverErrors, Instruction, ID},
    response::Response,
    serial,
};
use serialport::SerialPort;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServoError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("IO Error: {0}")]
    Driver(#[from] DriverErrors),
    #[error("A response was expected, but none received.")]
    NoResponse,
}

pub struct Servo {
    id: ID,
}

impl Servo {
    pub fn new(id: ID) -> Self {
        Self { id }
    }

    pub fn ping(&self, port: &mut Box<dyn SerialPort>) -> Result<Response, ServoError> {
        let ping = Command::new(self.id, Instruction::Ping);

        expect_response(serial::packet_tx_rx(ping, port))
    }

    pub fn assign_id(
        &mut self,
        new_id: ID,
        port: &mut Box<dyn SerialPort>,
    ) -> Result<Response, ServoError> {
        let instruction = Instruction::write_single(Address::ID, new_id.value())
            .expect("ID Address must be writeable.");
        let command = Command::new(self.id, instruction);
        let response = expect_response(serial::packet_tx_rx(command, port))?;

        self.id = new_id;

        Ok(response)
    }
}

fn expect_response(value: Result<Option<Response>, io::Error>) -> Result<Response, ServoError> {
    match value {
        Err(e) => Err(e.into()),
        Ok(None) => Err(ServoError::NoResponse),
        Ok(Some(x)) => Ok(x),
    }
}
