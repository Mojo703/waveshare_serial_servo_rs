use std::io;

use crate::{
    command::Command,
    hardware::{Address, AddressRegion, DriverErrors, Instruction, InstructionError, ID},
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

#[derive(Debug, Clone)]
pub struct Assign([Option<u8>; Assign::MAX_ADDRESS]);

impl Default for Assign {
    fn default() -> Self {
        Self([None; Assign::MAX_ADDRESS])
    }
}

impl Assign {
    const MAX_ADDRESS: usize = 56;

    pub fn set_position_goal(position: Position, speed: Speed, acceleration: Acceleration) -> Self {
        let mut order = Self::default();
        order.set_acceleration(Some(acceleration));
        order.set_position(Some(position));
        order.set_speed(Some(speed));

        // Set to make memory continuous.
        order.set_raw(Address::GoalTimeL, Some(0));
        order.set_raw(Address::GoalTimeH, Some(0));

        order
    }

    pub fn set_acceleration(&mut self, acceleration: Option<Acceleration>) {
        self.set_raw(Address::Acceleration, acceleration.map(|a| a.0));
    }

    pub fn set_position(&mut self, position: Option<Position>) {
        let (position_l, position_h) = split_word(position.map(|p| p.0));

        self.set_raw(Address::GoalPositionL, position_l);
        self.set_raw(Address::GoalPositionH, position_h);
    }

    pub fn set_speed(&mut self, speed: Option<Speed>) {
        let (speed_l, speed_h) = split_word(speed.map(|s| s.0));

        self.set_raw(Address::GoalSpeedL, speed_l);
        self.set_raw(Address::GoalSpeedH, speed_h);
    }

    fn set_raw(&mut self, address: Address, value: Option<u8>) {
        self.0[address as usize] = value;
    }

    fn get_raw(&self, address: Address) -> Option<u8> {
        self.0[address as usize]
    }

    fn get_instructions(&self) -> Vec<Result<Instruction, InstructionError>> {
        // Collect the addresses into contiguous instructions
        self.0
            .iter()
            .copied()
            .chain(std::iter::once(None))
            .enumerate()
            .fold(
                (Vec::new(), None::<(usize, Vec<u8>)>),
                |(mut instructions, collected), (index, value)| match (collected, value) {
                    (Some((start, mut collected)), Some(value)) => {
                        collected.push(value);
                        (instructions, Some((start, collected)))
                    }
                    (None, Some(value)) => (instructions, Some((index, Vec::<u8>::from([value])))),
                    (None, None) => (instructions, None),
                    (Some((start, collected)), None) => {
                        let start = Address::n(start as u8)
                            .expect("as_memory must store valid memory addresses.");
                        instructions.push(Instruction::write(start, collected));
                        (instructions, None)
                    }
                },
            )
            .0
    }
}

fn split_word(word: Option<u16>) -> (Option<u8>, Option<u8>) {
    match word {
        Some(value) => {
            let [l, h] = value.to_le_bytes();
            (Some(l), Some(h))
        }
        None => (None, None),
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum PropertyError {
    #[error("The property is out of range.")]
    OutOfRange,
}

#[derive(Debug, Clone, Copy)]
pub struct Speed(u16);

impl Speed {
    const MIN: u16 = 0;
    const MAX: u16 = 0xfff;

    pub fn new(value: f32) -> Self {
        let value = (value * (Self::MAX - Self::MIN) as f32) as u16 + Self::MIN;
        Self(value)
    }

    pub fn new_raw(value: u16) -> Result<Self, PropertyError> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then(|| Self(value))
            .ok_or(PropertyError::OutOfRange)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position(u16);

impl Position {
    const MIN: u16 = 0;
    const MAX: u16 = 0xfff;

    pub fn new<T: Angle<f32>>(position: T) -> Self {
        let position = ((position.to_deg().as_value() * 4096.0 / 360.0).round() as u16)
            .clamp(Self::MIN, Self::MAX);
        Self(position)
    }

    pub fn new_raw(value: u16) -> Result<Self, PropertyError> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then(|| Self(value))
            .ok_or(PropertyError::OutOfRange)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Acceleration(u8);

impl Acceleration {
    const MIN: u8 = 0;
    const MAX: u8 = 254;

    pub fn new(acceleration: f32) -> Self {
        let acceleration = (acceleration * (Self::MAX - Self::MIN) as f32) as u8 + Self::MIN;
        Self(acceleration)
    }

    pub fn new_raw(value: u8) -> Result<Self, PropertyError> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then(|| Self(value))
            .ok_or(PropertyError::OutOfRange)
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

    pub fn write_id(
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

    pub fn write(&self, assign: &Assign, port: &mut Box<dyn SerialPort>) -> Result<(), ServoError> {
        for instruction in assign.get_instructions() {
            let instruction = instruction?;
            let command = Command::new(self.id, instruction);
            serial::packet_tx_rx(command, port)?;
        }
        Ok(())
    }
}

fn expect_response(value: Result<Option<Response>, io::Error>) -> Result<Response, ServoError> {
    match value {
        Err(e) => Err(e.into()),
        Ok(None) => Err(ServoError::NoResponse),
        Ok(Some(x)) => Ok(x),
    }
}
