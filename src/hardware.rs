use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IDError {
	#[error("ID value out of allowed range.")]
	Range,
	#[error("ID cannot be broadcast.")]
	MustNotBroadcast,
	#[error("ID must be broadcast.")]
	MustBroadcast,
}

#[derive(Debug, Clone, Copy)]
pub enum ID {
	Broadcast,
	Single(u8),
}

impl ID {
	const BROADCAST: u8 = 0xfe;
	const MAX: u8 = 0xfc;

	pub fn broadcast() -> Self {
		Self::Broadcast
	}

	pub fn single(value: u8) -> Result<Self, IDError> {
		match value {
			Self::BROADCAST => Err(IDError::MustNotBroadcast),
			0..=Self::MAX => Ok(Self::Single(value)),
			_ => Err(IDError::Range)
		}
	}

	pub fn all_single() -> std::iter::FilterMap<std::ops::RangeInclusive<u8>, impl FnMut(u8) -> Option<ID>> {
		(0..=Self::MAX).filter_map(|value| Self::single(value).ok())
	}

	pub fn value(self) -> u8 {
		match self {
			Self::Broadcast => Self::BROADCAST,
			Self::Single(x) => x.into()
		}
	}
}

impl From<ID> for u8 {
	fn from(id: ID) -> Self {
		id.value()
	}
}

impl TryFrom<u8> for ID {
	type Error = IDError;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		if value == Self::BROADCAST {
			return Ok(Self::Broadcast)
		}

		Self::single(value)
	}
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, enumn::N, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Address {
    // EEPROM. Read only.
    ModelL = 3,
    ModelH = 4,

    // EEPROM. Read or Write.
    ID = 5,
    BaudRate = 6,
    MinAngleLimitL = 9,
    MinAngleLimitH = 10,
    MaxAngleLimitL = 11,
    MaxAngleLimitH = 12,
    CwDead = 26,
    CcwDead = 27,
    OfsL = 31,
    OfsH = 32,
    Mode = 33,

    // SRAM. Read or Write.
    TorqueEnable = 40,
    Acceleration = 41,
    GoalPositionL = 42,
    GoalPositionH = 43,
    GoalTimeL = 44,
    GoalTimeH = 45,
    GoalSpeedL = 46,
    GoalSpeedH = 47,
    Lock = 55,

    // SRAM. Read only.
    PresentPositionL = 56,
    PresentPositionH = 57,
    PresentSpeedL = 58,
    PresentSpeedH = 59,
    PresentLoadL = 60,
    PresentLoadH = 61,
    PresentVoltage = 62,
    PresentTemperature = 63,
    Moving = 66,
    PresentCurrentL = 69,
    PresentCurrentH = 70,
}

impl Address {
	fn can_write(self) -> bool {
		use Address::*;

		matches!(self,
			// EEPROM
            ID | BaudRate | MinAngleLimitL | MinAngleLimitH | MaxAngleLimitL | MaxAngleLimitH
            | CwDead | CcwDead | OfsL | OfsH | Mode | 
			// SRAM
			TorqueEnable | Acceleration | GoalPositionL
            | GoalPositionH | GoalTimeL | GoalTimeH | GoalSpeedL | GoalSpeedH | Lock)
	}

	fn can_read(self) -> bool {
		true
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AddressRegion {
    start: Address,
    length: u8,
}

impl AddressRegion {
    pub fn new(start: Address, length: u8) -> Self {
        Self { start, length }
    }

    pub fn one(address: Address) -> Self {
        let start = address;
        let length = 1;

        Self { start, length }
    }

    pub fn contains(self, value: Address) -> bool {
        let start = self.start as u8;
        let value = value as u8;
        let end = start + self.length;
        (start..end).contains(&value)
    }

    fn can_read(self) -> bool {
		self.iter().all(|address| address.can_read())
    }

    fn can_write(self) -> bool {
		self.iter().all(|address| address.can_write())
    }

	fn iter(&self) -> AddressRegionIter {
		AddressRegionIter::new(self.start, self.length)
	}
}

struct AddressRegionIter {
	last: u8,
	index: u8,
}

impl AddressRegionIter {
	fn new(start: Address, length: u8) -> Self {
		let first = start as u8;
		let last = first + length - 1;
		let index = first;
		Self { last, index }
	}
}

impl Iterator for AddressRegionIter {
	type Item = Address;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if self.index > self.last {
				return None;
			}
			
			let address = Address::n(self.index);
			self.index += 1;
			if let Some(next) = address {
				return Some(next);
			}
		}
	}
}

#[derive(Debug, Clone, Copy, Error)]
pub struct DriverErrors {
	errors: u8,
}

impl Display for DriverErrors {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let errors: Vec<String> = DriverError::VALUES.into_iter().filter(|&error| self.contains(error)).map(|error| format!("{error}")).collect();
		write!(f, "DriverErrors: {errors:?}")
	}
}

impl DriverErrors {
	fn contains(self, error: DriverError) -> bool {
		self.errors & error as u8 > 0
	}

	pub(crate) fn from_byte(value: u8) -> Option<Self> {
		let errors = Self {
			errors: value,
		};

		if !DriverError::VALUES.into_iter().any(|error| errors.contains(error)) {
			return None;
		}

		Some(errors)
	}
}

#[repr(u8)]
#[derive(Debug, Error, Clone, Copy)]
pub enum DriverError {
    #[error("Input voltage out of allowed range.")]
    Voltage = 1,
    #[error("Received angle is invalid.")]
    Angle = 2,
    #[error("Driver overheat.")]
    OverHeat = 4,
    #[error("Driver overele.")]
    OverEle = 8,
    #[error("Driver overload.")]
    OverLoad = 32,
}

impl DriverError {
    const VALUES: [Self; 5] = [
        Self::Voltage,
        Self::Angle,
        Self::OverHeat,
        Self::OverEle,
        Self::OverLoad,
    ];
}

#[derive(Debug, Error)]
pub enum InstructionError {
    #[error("Address region is not avaliable for read.")]
    ReadRegion,
    #[error("Address region is not avaliable for write.")]
    WriteRegion,
    #[error("The data length is invalid.")]
    DataLength,
}

#[derive(Debug)]
pub enum Instruction {
    Ping,
    Read(AddressRegion),
    Write { start: Address, data: Vec<u8> },
}

impl Instruction {
	pub fn ping() -> Instruction {
		Instruction::Ping
	}

    pub fn read(region: AddressRegion) -> Result<Instruction, InstructionError> {
        if region.can_read() {
            Ok(Instruction::Read(region))
        } else {
            Err(InstructionError::ReadRegion)
        }
    }

	pub fn write_single(address: Address, value: u8) -> Result<Instruction, InstructionError> {
		Self::write(address, Vec::from([value]))
	}

    pub fn write(start: Address, data: Vec<u8>) -> Result<Instruction, InstructionError> {
        let Ok(length) = data.len().try_into() else {
            return Err(InstructionError::DataLength);
        };

        let region = AddressRegion::new(start, length);
        if region.can_write() {
            Ok(Instruction::Write { start, data })
        } else {
            Err(InstructionError::WriteRegion)
        }
    }

    pub(crate) fn data(self) -> Vec<u8> {
        match self {
            Self::Ping => Vec::from([0x01]),
            Self::Read(region) => Vec::from([0x02, region.start as u8, region.length]),
            Self::Write { start, data } => {
                Vec::from_iter([0x03, start as u8].into_iter().chain(data))
            }
        }
    }
}
