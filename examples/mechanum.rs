use std::io::stdin;

use serialport::SerialPort;
use waveshare_serial_servo::{
    hardware::{address, ID},
    servo::{Acceleration, Assign, Mode, Servo, ServoError, Speed},
};

#[path = "./common/lib.rs"]
mod common;

struct Body {
    front_left: Servo,
    front_right: Servo,
    back_left: Servo,
    back_right: Servo,
}

struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Body {
    fn new(
        front_left_id: ID,
        front_right_id: ID,
        back_left_id: ID,
        back_right_id: ID,
        port: &mut Box<dyn SerialPort>,
    ) -> Result<Self, ServoError> {
        let body = Self {
            front_left: Servo::new(front_left_id),
            front_right: Servo::new(front_right_id),
            back_left: Servo::new(back_left_id),
            back_right: Servo::new(back_right_id),
        };

        body.ping_all(port)?;

        body.set_wheel_all(port)?;

        Ok(body)
    }

    fn ping_all(&self, port: &mut Box<dyn SerialPort>) -> Result<(), ServoError> {
        self.front_left.ping(port)?;
        self.front_right.ping(port)?;
        self.back_left.ping(port)?;
        self.back_right.ping(port)?;

        Ok(())
    }

    fn set_wheel_all(&self, port: &mut Box<dyn SerialPort>) -> Result<(), ServoError> {
        let wheel_mode = Assign::new().with(Mode::Wheel);

        self.front_left.write(&wheel_mode, port)?;
        self.front_right.write(&wheel_mode, port)?;
        self.back_left.write(&wheel_mode, port)?;
        self.back_right.write(&wheel_mode, port)?;

        Ok(())
    }

    fn set_movement(
        &self,
        velocity: Vec2,
        rotation: f32,
        port: &mut Box<dyn SerialPort>,
    ) -> Result<(), ServoError> {
        let front_left = velocity.x + velocity.y + rotation;
        let front_right = -velocity.x + velocity.y - rotation;
        let back_left = -velocity.x + velocity.y + rotation;
        let back_right = velocity.x + velocity.y - rotation;

        let mut assign = Assign::new().with(Acceleration::new(1.0));

        // Make memory continuous to lower packet count.
        assign.set_word(address::GoalPosition, Some(0));
        assign.set_word(address::GoalTime, Some(0));

        // Send commands to each servo.
        assign.set(Speed::new(front_left));
        self.front_left.write(&assign, port)?;

        assign.set(Speed::new(front_right));
        self.front_right.write(&assign, port)?;

        assign.set(Speed::new(back_left));
        self.back_left.write(&assign, port)?;

        assign.set(Speed::new(back_right));
        self.back_right.write(&assign, port)?;

        Ok(())
    }
}

fn ask_movement() -> Option<(Vec2, f32)> {
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("stdin read_line must work.");

    let parts: Vec<f32> = input
        .split(" ")
        .filter_map(|part| part.trim().parse().ok())
        .collect();

    if parts.len() != 3 {
        return None;
    }

    let vec = Vec2::new(parts[0], parts[1]);

    Some((vec, parts[2]))
}

fn main() {
    let mut port = common::get_port();

    let front_left_id = ID::single(1).unwrap();
    let front_right_id = ID::single(2).unwrap();
    let back_left_id = ID::single(3).unwrap();
    let back_right_id = ID::single(4).unwrap();
    let body = Body::new(
        front_left_id,
        front_right_id,
        back_left_id,
        back_right_id,
        &mut port,
    )
    .expect("Body must be set up with valid servos.");

    println!("Provide a vector and rotation.");

    loop {
        let Some((vec, rot)) = ask_movement() else {
            continue;
        };

        if let Err(e) = body.set_movement(vec, rot, &mut port) {
            println!("body.set_movement Error: {e}");
        }
    }
}
