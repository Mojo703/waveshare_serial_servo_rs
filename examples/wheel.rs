/// Ping the serial bus for any devices.
///
extern crate waveshare_serial_servo;

#[path = "./common/lib.rs"]
mod common;

use std::io::stdin;

use waveshare_serial_servo::{
    hardware::address,
    servo::{Acceleration, Assign, Mode, Servo, Speed},
};

fn ask_speed() -> Option<f32> {
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
        .write(&Assign::new().with(Mode::Wheel), &mut port)
        .expect("Servo write wheel mode must work.");

    let mut assign = Assign::new().with(Acceleration::new(1.0));

    // Make memory continuous to lower packet count.
    assign.set_word(address::GoalPosition, Some(0));
    assign.set_word(address::GoalTime, Some(0));

    loop {
        let Some(speed) = ask_speed() else {
            continue;
        };
        assign.set(Speed::new(speed));
        println!("response: {:?}", servo.write(&assign, &mut port));
    }
}
