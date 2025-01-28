/// Ping the serial bus for any devices.
///
extern crate waveshare_serial_servo;

#[path = ".\\common\\lib.rs"]
mod common;

use waveshare_serial_servo::{hardware::ID, servo::Servo};

fn main() {
    let mut port = common::get_port();

    println!("Searching for any devices: ");

    let found_count = ID::all_single()
        .filter(|&id| {
            if let Ok(response) = Servo::new(id).ping(&mut port) {
                println!("Received from ID {:#04x}: {response:?}", id.value());
                true
            } else {
                false
            }
        })
        .count();

    println!("Finished. Found {} device(s).", found_count);
}
