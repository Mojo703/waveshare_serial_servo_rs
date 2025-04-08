/// Ping the serial bus for any devices.
///
extern crate waveshare_serial_servo;

#[path = "./common/lib.rs"]
mod common;

use std::{io::stdin, str::FromStr, thread, time::Duration};

use serialport::SerialPort;
use waveshare_serial_servo::{
    hardware::address::{self, ReadRegion},
    servo::{Acceleration, Assign, Mode, Position, Servo, Speed},
};

fn ask<T: FromStr>() -> Option<T> {
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("stdin read_line must work.");
    input.trim().parse().ok()
}

fn read_position(servo: &Servo, port: &mut Box<dyn SerialPort>) -> u8 {
    servo
        .read(ReadRegion::one(address::PresentPosition), port)
        .unwrap()
        .payload[0]
}

fn linear_to_angle(linear: f32) -> Position {
    // Approximate as a linear mapping (40.0 -> 0, 2.0 -> 1000)

    let pos = 1000.0 - (linear - 2.0) / (40.0 - 2.0) * 1000.0;
    let pos = (pos as u16).clamp(0, 1000);

    Position::new_raw(pos).expect("linear_to_angle must be provided a valid position")
}

fn main() {
    // 80 to 52
    let mut port = common::get_port();

    println!("Servo ID?");
    let id = common::ask_id();

    let servo = Servo::new(id);

    servo
        .write(&Assign::new().with(Mode::Position), &mut port)
        .expect("Servo write position mode must work.");

    let mut assign = Assign::new().with(Acceleration::new(0.1));

    loop {
        print!("Linear position (mm): ");
        let Some(linear) = ask() else {
            println!("invalid linear position");
            continue;
        };

        let pos = linear_to_angle(linear);
        println!("linear: {linear}, pos: {pos:?}");

        assign.set(pos);

        println!("response: {:?}", servo.write(&assign, &mut port));
    }

    // servo
    //     .write(&Assign::new().with(Mode::Wheel), &mut port)
    //     .expect("Servo write wheel mode must work.");

    // let mut assign = Assign::new().with(Acceleration::new(1.0));

    // // Make memory continuous to lower packet count.
    // assign.set_word(address::GoalPosition, Some(0));
    // assign.set_word(address::GoalTime, Some(0));

    // loop {
    //     let Some(value) = ask::<u32>() else {
    //         continue;
    //     };
    //     let speed = Speed::new_raw_unchecked(value as u16);
    //     println!("speed {value} -> {:?}", speed);
    //     assign.set(speed);
    //     println!("response: {:?}", servo.write(&assign, &mut port));

    //     break;
    // }

    // // let start = read_position(&servo, &mut port);

    // for _ in 0..1000 {
    //     println!(
    //         "position: {}",
    //         servo
    //             .read(ReadRegion::one(address::PresentPosition), &mut port)
    //             .unwrap()
    //             .payload[0]
    //     );

    //     thread::sleep(Duration::from_millis(10));
    // }

    // assign.set(Speed::new_raw_unchecked(0));
    // servo.write(&assign, &mut port).unwrap();
}
