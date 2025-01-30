/// Ping the serial bus for any devices.
///
extern crate waveshare_serial_servo;

#[path = ".\\common\\lib.rs"]
mod common;

use std::{thread::sleep, time::Duration};

use angle::Deg;
use waveshare_serial_servo::servo::{MoveConfig, Servo};

fn main() {
    let mut port = common::get_port();

    println!("Servo ID?");
    let id = common::ask_id();

    let servo = Servo::new(id);

    println!(
        "Ping response: {:?}",
        servo.ping(&mut port).expect("Servo must be avaliable.")
    );

    let config_a = MoveConfig {
        acceleration: 0xff - 1,
        position: 0x0000,
        speed: 0x0fff,
    };

    let config_b = config_a.with_position(Deg(360.0));

    loop {
        println!("response: {:?}", servo.write_move(config_a, &mut port));
        sleep(Duration::from_secs(3));
        println!("response: {:?}", servo.write_move(config_b, &mut port));
        sleep(Duration::from_secs(3));
    }
}
