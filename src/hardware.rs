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
            _ => Err(IDError::Range),
        }
    }

    pub fn all_single(
    ) -> std::iter::FilterMap<std::ops::RangeInclusive<u8>, impl FnMut(u8) -> Option<ID>> {
        (0..=Self::MAX).filter_map(|value| Self::single(value).ok())
    }

    pub fn value(self) -> u8 {
        match self {
            Self::Broadcast => Self::BROADCAST,
            Self::Single(x) => x.into(),
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
            return Ok(Self::Broadcast);
        }

        Self::single(value)
    }
}

pub mod address {
    macro_rules! address {
        ($name:ident, $value:expr, Byte, Read) => {
            pub struct $name;

            impl Address for $name {
                fn index(&self) -> u8 {
                    $value
                }

                fn size(&self) -> Size {
                    Size::Byte
                }

                fn access(&self) -> Access {
                    Access::Read
                }
            }

            impl ReadableAddress for $name {}
            impl ByteAddress for $name {}
        };
        ($name:ident, $value:expr, Byte, ReadWrite) => {
            pub struct $name;

            impl Address for $name {
                fn index(&self) -> u8 {
                    $value
                }

                fn size(&self) -> Size {
                    Size::Byte
                }

                fn access(&self) -> Access {
                    Access::ReadWrite
                }
            }

            impl ReadableAddress for $name {}
            impl WriteableAddress for $name {}
            impl ByteAddress for $name {}
        };
        ($name:ident, $value:expr, Word, Read) => {
            pub struct $name;

            impl Address for $name {
                fn index(&self) -> u8 {
                    $value
                }

                fn size(&self) -> Size {
                    Size::Word
                }

                fn access(&self) -> Access {
                    Access::Read
                }
            }

            impl ReadableAddress for $name {}
            impl WordAddress for $name {}
        };
        ($name:ident, $value:expr, Word, ReadWrite) => {
            pub struct $name;

            impl Address for $name {
                fn index(&self) -> u8 {
                    $value
                }

                fn size(&self) -> Size {
                    Size::Word
                }

                fn access(&self) -> Access {
                    Access::ReadWrite
                }
            }

            impl ReadableAddress for $name {}
            impl WriteableAddress for $name {}
            impl WordAddress for $name {}
        };
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Size {
        Byte,
        Word,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Access {
        Read,
        ReadWrite,
    }

    impl Access {
        pub fn can_write(self) -> bool {
            match self {
                Self::Read => false,
                Self::ReadWrite => true,
            }
        }

        pub fn can_read(self) -> bool {
            true
        }
    }

    pub trait Address {
        fn index(&self) -> u8;

        fn size(&self) -> Size;

        fn access(&self) -> Access;
    }

    pub trait WriteableAddress: Address {}
    pub trait ReadableAddress: Address {}
    pub trait ByteAddress: Address {}
    pub trait WordAddress: Address {
        fn index_l(&self) -> u8 {
            self.index()
        }

        fn index_h(&self) -> u8 {
            self.index() + 1
        }
    }

    // EEPROM. Read only.
    address!(Model, 3, Word, Read);

    // EEPROM. Read or Write.
    address!(ID, 5, Byte, ReadWrite);
    address!(BaudRate, 6, Byte, ReadWrite);
    address!(MinAngleLimit, 9, Word, ReadWrite);
    address!(MaxAngleLimit, 11, Word, ReadWrite);
    address!(CwDead, 26, Byte, ReadWrite);
    address!(CcwDead, 27, Byte, ReadWrite);
    address!(Ofs, 31, Word, ReadWrite);
    address!(Mode, 33, Byte, ReadWrite);

    // SRAM. Read or Write.
    address!(TorqueEnable, 40, Byte, ReadWrite);
    address!(Acceleration, 41, Byte, ReadWrite);
    address!(GoalPosition, 42, Word, ReadWrite);
    address!(GoalTime, 44, Word, ReadWrite);
    address!(GoalSpeed, 46, Word, ReadWrite);
    address!(Lock, 55, Byte, ReadWrite);

    // SRAM. Read only.
    address!(PresentPosition, 56, Word, Read);
    address!(PresentSpeed, 58, Word, Read);
    address!(PresentLoad, 60, Word, Read);
    address!(PresentVoltage, 62, Byte, Read);
    address!(PresentTemperature, 63, Byte, Read);
    address!(Moving, 66, Byte, Read);
    address!(PresentCurrent, 69, Word, Read);

    pub fn address_from(value: u8) -> Option<Box<dyn Address>> {
        Some(match value {
            3 | 4 => Box::new(Model),
            5 => Box::new(ID),
            6 => Box::new(BaudRate),
            9 | 10 => Box::new(MinAngleLimit),
            11 | 12 => Box::new(MaxAngleLimit),
            26 => Box::new(CwDead),
            27 => Box::new(CcwDead),
            31 | 32 => Box::new(Ofs),
            33 => Box::new(Mode),
            40 => Box::new(TorqueEnable),
            41 => Box::new(Acceleration),
            42 | 43 => Box::new(GoalPosition),
            44 | 45 => Box::new(GoalTime),
            46 | 47 => Box::new(GoalSpeed),
            55 => Box::new(Lock),
            56 | 57 => Box::new(PresentPosition),
            58 | 59 => Box::new(PresentSpeed),
            60 | 61 => Box::new(PresentLoad),
            62 => Box::new(PresentVoltage),
            63 => Box::new(PresentTemperature),
            66 => Box::new(Moving),
            69 | 70 => Box::new(PresentCurrent),
            _ => return None,
        })
    }

    pub fn writeable_address_from(value: u8) -> Option<Box<dyn WriteableAddress>> {
        Some(match value {
            5 => Box::new(ID),
            6 => Box::new(BaudRate),
            9 | 10 => Box::new(MinAngleLimit),
            11 | 12 => Box::new(MaxAngleLimit),
            26 => Box::new(CwDead),
            27 => Box::new(CcwDead),
            31 | 32 => Box::new(Ofs),
            33 => Box::new(Mode),
            40 => Box::new(TorqueEnable),
            41 => Box::new(Acceleration),
            42 | 43 => Box::new(GoalPosition),
            44 | 45 => Box::new(GoalTime),
            46 | 47 => Box::new(GoalSpeed),
            55 => Box::new(Lock),
            _ => return None,
        })
    }

    #[derive(Debug, Clone)]
    pub struct WriteRegion {
        pub(crate) start: u8,
        pub(crate) data: Vec<u8>,
    }

    impl WriteRegion {
        pub fn new(start: u8, data: Vec<u8>) -> Option<Self> {
            let end = start + data.len() as u8;
            (start..end)
                .all(|i| address_from(i).is_some_and(|a| a.access().can_write()))
                .then(|| Self { start, data })
        }

        pub fn one<A: WriteableAddress>(address: A, value: u8) -> Self {
            Self {
                start: address.index(),
                data: Vec::from([value]),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ReadRegion {
        pub(crate) start: u8,
        pub(crate) length: u8,
    }

    impl ReadRegion {
        pub fn new(start: u8, length: u8) -> Option<Self> {
            let end = start + length;
            (start..end)
                .all(|i| address_from(i).is_some_and(|a| a.access().can_read()))
                .then(|| Self { start, length })
        }

        pub fn one<W: ReadableAddress>(address: W) -> Self {
            Self {
                start: address.index(),
                length: 1,
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
        let errors: Vec<String> = DriverError::VALUES
            .into_iter()
            .filter(|&error| self.contains(error))
            .map(|error| format!("{error}"))
            .collect();
        write!(f, "DriverErrors: {errors:?}")
    }
}

impl DriverErrors {
    fn contains(self, error: DriverError) -> bool {
        self.errors & error as u8 > 0
    }

    pub(crate) fn from_byte(value: u8) -> Option<Self> {
        let errors = Self { errors: value };

        if !DriverError::VALUES
            .into_iter()
            .any(|error| errors.contains(error))
        {
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

#[derive(Debug)]
pub enum Instruction {
    Ping,
    Read(address::ReadRegion),
    Write(address::WriteRegion),
}

impl Instruction {
    pub fn ping() -> Instruction {
        Self::Ping
    }

    pub fn read(region: address::ReadRegion) -> Self {
        Self::Read(region)
    }

    pub fn write(region: address::WriteRegion) -> Self {
        Self::Write(region)
    }

    pub(crate) fn data(self) -> Vec<u8> {
        match self {
            Self::Ping => Vec::from([0x01]),
            Self::Read(region) => Vec::from([0x02, region.start as u8, region.length]),
            Self::Write(region) => {
                Vec::from_iter([0x03, region.start as u8].into_iter().chain(region.data))
            }
        }
    }
}
