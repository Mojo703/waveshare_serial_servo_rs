/// Ping the serial bus for any devices.
///
extern crate waveshare_serial_servo;

#[path = ".\\common\\lib.rs"]
mod common;

use std::io::stdin;

use waveshare_serial_servo::{hardware::ID, servo::Servo};

fn ask_id() -> ID {
    loop {
        print!("ID: ");
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .expect("stdin read_line must work.");
        let Ok(value) = input.trim().parse::<u8>() else {
            println!("Invalid: not a unsigned 8bit integer.");
            continue;
        };
        match ID::try_from(value) {
            Err(e) => {
                println!("Invalid: not a valid id. Error: {e}");
                continue;
            }
            Ok(id) => return id,
        }
    }
}

fn main() {
    let mut port = common::get_port();

    println!("Current ID?");
    let current = ask_id();

    println!("New ID?");
    let new = ask_id();

    let mut servo = Servo::new(current);

    let response = servo.ping(&mut port).expect("Servo must be avaliable.");
    println!("Ping response: {response:?}");

    let response = servo
        .assign_id(new, &mut port)
        .expect("Servo must be avaliable.");
    println!("Assign response: {response:?}");

    let response = servo.ping(&mut port).expect("Servo must be avaliable.");
    println!("New ping response: {response:?}");
}
