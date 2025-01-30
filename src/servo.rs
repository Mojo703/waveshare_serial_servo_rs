use std::io;

use crate::{
    command::Command,
    hardware::{Address, DriverErrors, Instruction, ID},
    response::Response,
    serial,
};
use angle::Angle;
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

#[derive(Debug, Clone, Copy)]
pub struct MoveConfig {
    /// Valid range [0, 254]
    pub acceleration: u8,
    /// Valid range [0, 4096) maps to [0, 360) degrees
    pub position: u16,
    /// Valid range [0, 4096)
    pub speed: u16,
}

impl MoveConfig {
    pub fn with_acceleration(mut self, acceleration: u8) -> Self {
        self.acceleration = acceleration;
        self
    }

    pub fn with_position_raw(mut self, position: u16) -> Self {
        self.position = position;
        self
    }

    pub fn with_position<T: Angle<f32>>(self, angle: T) -> Self {
        let position = (angle.to_deg().as_value() * 4096.0 / 360.0)
            .clamp(0.0, 4095.0)
            .round() as u16;
        self.with_position_raw(position)
    }

    pub fn with_speed(mut self, speed: u16) -> Self {
        self.speed = speed;
        self
    }

    fn as_write(self) -> Instruction {
        let MoveConfig {
            acceleration,
            position,
            speed,
        } = self;
        let [position_l, position_h] = position.to_le_bytes();
        let [speed_l, speed_h] = speed.to_le_bytes();

        let start = Address::Acceleration;
        let data = Vec::from([acceleration, position_l, position_h, 0, 0, speed_l, speed_h]);

        Instruction::write(start, data).expect("MoveConfig as_write must be valid.")
    }
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

    pub fn write_move(
        &self,
        config: MoveConfig,
        port: &mut Box<dyn SerialPort>,
    ) -> Result<Response, ServoError> {
        let command = Command::new(self.id, config.as_write());
        expect_response(serial::packet_tx_rx(command, port))
    }
}

fn expect_response(value: Result<Option<Response>, io::Error>) -> Result<Response, ServoError> {
    match value {
        Err(e) => Err(e.into()),
        Ok(None) => Err(ServoError::NoResponse),
        Ok(Some(x)) => Ok(x),
    }
}
