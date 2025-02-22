use crate::{
    command::Command,
    hardware::{
        address::{self, WriteRegion},
        DriverErrors, Instruction, ID,
    },
    response::Response,
    serial::{self, SerialError},
};
use angle::Angle;
use serialport::SerialPort;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServoError {
    #[error("Serial Error: {0}")]
    Serial(#[from] SerialError),
    #[error("Driver Error: {0}")]
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
        order.set_word(address::GoalTime, Some(0));

        order
    }

    pub fn set_acceleration(&mut self, acceleration: Option<Acceleration>) {
        self.set_byte(address::Acceleration, acceleration.map(|a| a.0));
    }

    pub fn set_position(&mut self, position: Option<Position>) {
        self.set_word(address::GoalPosition, position.map(|p| p.0));
    }

    pub fn set_speed(&mut self, speed: Option<Speed>) {
        self.set_word(address::GoalSpeed, speed.map(|s| s.0));
    }

    fn set_byte<A: address::ByteAddress>(&mut self, address: A, value: Option<u8>) {
        self.0[address.index() as usize] = value;
    }

    fn set_word<A: address::WordAddress>(&mut self, address: A, value: Option<u16>) {
        let (l, h) = split_word(value);
        self.0[address.index_l() as usize] = l;
        self.0[address.index_h() as usize] = h;
    }

    fn get_instructions(&self) -> Vec<Instruction> {
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
                    (None, Some(value)) => (instructions, Some((index, Vec::from([value])))),
                    (None, None) => (instructions, None),
                    (Some((start, collected)), None) => {
                        let region = address::WriteRegion::new(start as u8, collected)
                            .expect("as_memory must store valid memory regions");
                        instructions.push(Instruction::write(region));
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
        let region = WriteRegion::one(address::ID, new_id.value());
        let instruction = Instruction::write(region);
        let command = Command::new(self.id, instruction);

        self.write_eeprom_lock(false, port)?;
        let response = expect_response(serial::packet_tx_rx(command, port))?;
        self.id = new_id;
        self.write_eeprom_lock(true, port)?;

        Ok(response)
    }

    pub fn write(&self, assign: &Assign, port: &mut Box<dyn SerialPort>) -> Result<(), ServoError> {
        for instruction in assign.get_instructions() {
            let command = Command::new(self.id, instruction);
            serial::packet_tx_rx(command, port)?;
        }
        Ok(())
    }

    fn write_eeprom_lock(
        &self,
        locked: bool,
        port: &mut Box<dyn SerialPort>,
    ) -> Result<(), ServoError> {
        let locked = if locked { 1 } else { 0 };
        let region = WriteRegion::one(address::Lock, locked);
        let instruction = Instruction::write(region);
        let command = Command::new(self.id, instruction);
        serial::packet_tx_rx(command, port)?;

        Ok(())
    }
}

fn expect_response(value: Result<Option<Response>, SerialError>) -> Result<Response, ServoError> {
    match value {
        Err(e) => Err(e.into()),
        Ok(None) => Err(ServoError::NoResponse),
        Ok(Some(x)) => Ok(x),
    }
}
