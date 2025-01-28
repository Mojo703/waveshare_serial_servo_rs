use thiserror::Error;

use crate::{
    crc,
    hardware::{DriverError, DriverErrors, ID},
};

#[derive(Debug)]
pub struct Response {
    pub id: ID,
    pub errors: Option<DriverErrors>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("Response format is invalid.")]
    Malformed,
    #[error("CRC does not match.")]
    CrcInvalid,
    #[error("The id is not valid.")]
    IdInvalid,
}

impl TryFrom<&[u8]> for Response {
    type Error = ResponseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let id = value.get(2);
        let length = value.get(3);
        let error = value.get(4);

        // Make sure the packet has the required fields.
        let (Some(&id), Some(&length), Some(&error)) = (id, length, error) else {
            return Err(ResponseError::Malformed);
        };

        // Make sure the id is valid
        let Ok(id) = ID::try_from(id) else {
            return Err(ResponseError::IdInvalid);
        };

        // Make sure the crc is present.
        let Some(&rx_crc) = value.get(3 + length as usize) else {
            return Err(ResponseError::Malformed);
        };

        // Make sure the crc is valid.
        let crc = crc(&value[2..value.len() - 1]);
        if rx_crc != crc {
            return Err(ResponseError::CrcInvalid);
        }

        // Parse the errors.
        let error = DriverErrors::from_byte(error);

        // Collect the payload.
        let payload = Vec::from(&value[5..value.len() - 1]);

        Ok(Self {
            id,
            errors: error,
            payload,
        })
    }
}
