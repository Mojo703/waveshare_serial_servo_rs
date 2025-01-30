use std::{collections::HashMap, io};

use crate::{
    command::Command,
    hardware::{Address, DriverErrors, Instruction, InstructionError, ID},
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
    #[error("Driver Error: {0}")]
    Driver(#[from] DriverErrors),
    #[error("Instruction Error: {0}")]
    Instruction(#[from] InstructionError),
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

#[derive(Debug, Clone, Copy)]
pub struct Order {
    // /// Valid range [0, 254]
    // acceleration: Option<u8>,
    // /// Valid range [0, 4096) maps to [0, 360) degrees
    // position: Option<u16>,
    // /// Valid range [0, 4096)
    // speed: Option<u16>,
    values: HashMap<Address, u8>,
}

impl Order {
    pub fn with_acceleration(mut self, acceleration: Option<u8>) -> Self {
        self.acceleration = acceleration;
        self
    }

    pub fn with_position_raw(mut self, position: Option<u16>) -> Self {
        self.position = position;
        self
    }

    pub fn with_position<T: Angle<f32>>(self, angle: Option<T>) -> Self {
        let position = angle.map(|angle| {
            (angle.to_deg().as_value() * 4096.0 / 360.0)
                .clamp(0.0, 4095.0)
                .round() as u16
        });
        self.with_position_raw(position)
    }

    pub fn with_speed(mut self, speed: Option<u16>) -> Self {
        self.speed = speed;
        self
    }

    fn as_memory(self) -> [Option<u8>; 56] {
        let memory = [None; 56];

        memory
    }

    fn as_instructions(self) -> Vec<Result<Instruction, InstructionError>> {
        self.as_memory()
            .into_iter()
            .chain(std::iter::once(None))
            .enumerate()
            .fold(
                (Vec::new(), None::<Vec<u8>>),
                |(mut instructions, collected), (index, value)| match (collected, value) {
                    (Some(mut collected), Some(value)) => {
                        collected.push(value);
                        (instructions, Some(collected))
                    }
                    (None, Some(value)) => (instructions, Some(Vec::<u8>::from([value]))),
                    (None, None) => (instructions, None),
                    (Some(collected), None) => {
                        let start = Address::n(index as u8)
                            .expect("as_memory must store valid memory addresses.");
                        instructions.push(Instruction::write(start, collected.clone()));
                        (instructions, Some(collected))
                    }
                },
            )
            .0
    }
}

impl MoveConfig {
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

    pub fn write(&self, order: Order, port: &mut Box<dyn SerialPort>) -> Result<(), ServoError> {
        for instruction in order.as_instructions() {
            let instruction = instruction?;
            let command = Command::new(self.id, instruction);
            serial::packet_tx_rx(command, port)?;
        }
        Ok(())
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
