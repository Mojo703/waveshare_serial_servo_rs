/// Ping the serial bus for any devices.
///
extern crate waveshare_serial_servo;

#[path = "./common/lib.rs"]
mod common;

use std::{thread::sleep, time::Duration};

use angle::Deg;
use waveshare_serial_servo::servo::{Acceleration, Assign, Position, Servo, Speed};

fn main() {
    let mut port = common::get_port();

    println!("Servo ID?");
    let id = common::ask_id();

    let servo = Servo::new(id);

    // println!(
    //     "Ping response: {:?}",
    //     servo.ping(&mut port).expect("Servo must be avaliable.")
    // );

    let speed = Speed::new(1.0);
    let acceleration = Acceleration::new(1.0);
    let position_a = Assign::set_position_goal(Position::new(Deg(0.0)), speed, acceleration);
    let position_b = Assign::set_position_goal(Position::new(Deg(360.0)), speed, acceleration);

    loop {
        println!("response: {:?}", servo.write(&position_a, &mut port));
        sleep(Duration::from_secs(3));
        println!("response: {:?}", servo.write(&position_b, &mut port));
        sleep(Duration::from_secs(3));
    }
}
