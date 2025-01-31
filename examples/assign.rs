/// Ping the serial bus for any devices.
///
extern crate waveshare_serial_servo;

#[path = ".\\common\\lib.rs"]
mod common;

use waveshare_serial_servo::servo::Servo;

fn main() {
    let mut port = common::get_port();

    println!("Current ID?");
    let current = common::ask_id();

    println!("New ID?");
    let new = common::ask_id();

    let mut servo = Servo::new(current);

    let response = servo.ping(&mut port).expect("Servo must be avaliable.");
    println!("Ping response: {response:?}");

    let response = servo
        .write_id(new, &mut port)
        .expect("Servo must be avaliable.");
    println!("Assign response: {response:?}");

    let response = servo.ping(&mut port).expect("Servo must be avaliable.");
    println!("New ping response: {response:?}");
}
