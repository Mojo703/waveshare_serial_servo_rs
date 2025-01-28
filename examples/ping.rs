extern crate waveshare_serial_servo;

#[path = ".\\common\\lib.rs"]
mod common;

use waveshare_serial_servo::{
    command::Command,
    hardware::{Instruction, ID},
    serial::packet_tx_rx,
};

fn main() {
    let mut port = common::get_port();

    println!("Searching for any devices: ");

    let found_count = ID::all_single()
        .filter(|&id| {
            if let Ok(Some(ping_rx)) = packet_tx_rx(
                Command {
                    id,
                    instruction: Instruction::ping(),
                },
                &mut port,
            ) {
                println!("Received from ID {:#04x}: {ping_rx:?}", id.value());
                true
            } else {
                false
            }
        })
        .count();

    println!("Finished. Found {} device(s).", found_count);
}
