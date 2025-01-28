use serialport::SerialPort;

use crate::{command::Command, hardware::ID, response::Response};

pub fn packet_tx_rx(
    transmit: Command,
    port: &mut Box<dyn SerialPort>,
) -> Result<Option<Response>, std::io::Error> {
    let is_broadcast = matches!(transmit.id, ID::Broadcast);
    port.write_all(&transmit.build())?;

    // Device does not respond to broadcast, so do not listen for response.
    if is_broadcast {
        return Ok(None);
    }

    let mut receive = vec![0u8; 4]; // Minimum Packet [ HEADER0, HEADER1, ID, LENGTH ]
    port.read_exact(&mut receive)?;

    let mut remain = vec![0u8; receive[3] as usize];
    port.read_exact(&mut remain)?;
    receive.extend(remain);

    Ok(Some(
        Response::try_from(receive.as_slice()).expect("Received packet must be valid."),
    ))
}
