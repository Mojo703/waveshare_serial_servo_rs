/// Ping the serial bus for any devices.
///
extern crate waveshare_serial_servo;

#[path = "./common/lib.rs"]
mod common;

use std::io::stdin;

use waveshare_serial_servo::servo::{Acceleration, Assign, Mode, Position, Servo};

fn ask_position() -> Option<u16> {
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("stdin read_line must work.");
    input.trim().parse().ok()
}

fn main() {
    let mut port = common::get_port();

    println!("Servo ID?");
    let id = common::ask_id();

    let servo = Servo::new(id);

    servo
        .write(&Assign::new().with(Mode::Position), &mut port)
        .expect("Servo write position mode must work.");

    let mut assign = Assign::new().with(Acceleration::new(1.0));

    loop {
        let Some(value) = ask_position() else {
            continue;
        };
        let position = Position::new_raw(value).expect("Provided position must be valid.");
        println!("position {value} -> {:?}", position);
        assign.set(position);
        println!("response: {:?}", servo.write(&assign, &mut port));
    }
}
