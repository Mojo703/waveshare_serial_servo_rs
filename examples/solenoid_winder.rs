/// Ping the serial bus for any devices.
///
extern crate waveshare_serial_servo;

#[path = "./common/lib.rs"]
mod common;

use std::{io::stdin, str::FromStr, thread, time::Duration};

use waveshare_serial_servo::{
    hardware::address::{self, ReadRegion},
    servo::{Acceleration, Assign, Mode, Servo, Speed},
};

fn ask<T: FromStr>() -> Option<T> {
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
        let Some(value) = ask::<u32>() else {
            continue;
        };
        let speed = Speed::new_raw_unchecked(value as u16);
        println!("speed {value} -> {:?}", speed);
        assign.set(speed);
        println!("response: {:?}", servo.write(&assign, &mut port));

        break;
    }

    loop {
        println!(
            "position: {:?}",
            servo.read(ReadRegion::one(address::PresentPosition), &mut port)
        );

        thread::sleep(Duration::from_millis(100));
    }
}
