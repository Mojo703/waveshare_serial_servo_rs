pub mod command;
pub mod hardware;
pub mod response;
pub mod serial;
pub mod servo;

fn crc(packet: &[u8]) -> u8 {
    !packet.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte))
}
