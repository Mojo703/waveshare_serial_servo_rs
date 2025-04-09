/// Ping the serial bus for any devices.
///
extern crate waveshare_serial_servo;

#[path = "./common/lib.rs"]
mod common;

use std::time::Instant;

use serialport::SerialPort;
use waveshare_serial_servo::{
    hardware::address::{self, ReadRegion},
    servo::{Acceleration, Assign, Mode, Position, Servo, Speed},
};

fn read_position(servo: &Servo, port: &mut Box<dyn SerialPort>) -> u16 {
    let payload = servo
        .read(ReadRegion::one_word(address::PresentPosition), port)
        .unwrap()
        .payload;

    u16::from_le_bytes([payload[0], payload[1]])
}

type Millimeter = f32;

struct Settings<'a> {
    start: Millimeter,
    end: Millimeter,
    step: Millimeter,
    wraps: u32,

    rotation: &'a Servo,
    linear: &'a Servo,

    rotation_acceleration: Acceleration,
    rotation_speed: Speed, // Must be positive

    linear_map: &'a dyn Fn(f32) -> Position,

    port: &'a mut Box<dyn SerialPort>,
}

impl<'a> Settings<'a> {
    pub fn perform(&mut self) {
        let start = Instant::now();

        // Set up the servos
        self.rotation
            .write(
                &Assign::new()
                    .with(Mode::Wheel)
                    .with(self.rotation_acceleration)
                    .with(self.rotation_speed),
                self.port,
            )
            .expect("rotation servo setup must work.");
        self.linear
            .write(
                &Assign::new()
                    .with(Mode::Position)
                    .with(Speed::new(1.0))
                    .with(Acceleration::new(1.0)),
                self.port,
            )
            .expect("linear servo setup must work.");

        let mut loop_index: u32 = 0;

        let loops_per_layer = (self.end - self.start) / self.step;

        let mut previous_position = None;
        let mut previous_layer = None;

        loop {
            // Check if the rotation servo has looped around. Assume the direction is positive.
            let rotation_position = read_position(self.rotation, self.port);
            if previous_position
                .is_some_and(|previous_position| previous_position > rotation_position)
            {
                loop_index += 1;
            }
            previous_position = Some(rotation_position);
            let partial = rotation_position as f32 / Position::MAX as f32;

            // Make sure we have not reached the layer limit
            let layer_index = ((loop_index as f32 + partial) / loops_per_layer) as u32;
            if layer_index >= self.wraps {
                break;
            }
            if !previous_layer.is_some_and(|previous_layer| layer_index <= previous_layer) {
                println!(
                    "Started layer {}, with {} loops completed at {:.2?}.",
                    layer_index + 1,
                    loop_index + 1,
                    start.elapsed()
                );
            }
            previous_layer = Some(layer_index);

            // Update the linear arm
            let t = ((loop_index as f32 + partial) / loops_per_layer) % 1.0;
            let linear_position = match layer_index % 2 {
                0 => t * (self.end - self.start) + self.start,
                1 => (1.0 - t) * (self.end - self.start) + self.start,
                _ => unreachable!(),
            };
            self.linear
                .write(
                    &Assign::new().with((self.linear_map)(linear_position)),
                    self.port,
                )
                .expect("linear servo write position must work.");
        }

        println!(
            "Placed {} loop(s), over {} layer(s) in {:.2?}.",
            loop_index + 1,
            self.wraps,
            start.elapsed()
        );
    }
}

impl<'a> Drop for Settings<'a> {
    fn drop(&mut self) {
        self.rotation
            .write(&Assign::new().with(Speed::new_raw(0).unwrap()), self.port)
            .expect("rotation servo stop must work.");
    }
}

fn main() {
    let mut port = common::get_port();

    print!("Rotation servo ");
    let rotation = Servo::new(common::ask_id());
    print!("Linear servo ");
    let linear = Servo::new(common::ask_id());

    let mut settings = Settings {
        start: 7.0,
        end: 38.0,
        step: 0.45,
        wraps: 5,

        rotation_acceleration: Acceleration::new(1.0),
        rotation_speed: Speed::new(0.3),

        rotation: &rotation,
        linear: &linear,

        port: &mut port,

        linear_map: &|linear| {
            // Approximate as a linear mapping (40.0 -> 0, 2.0 -> 1000)
            let pos = 1000.0 - (linear - 2.0) / (40.0 - 2.0) * 1000.0;
            let pos = (pos as u16).clamp(0, 1000);

            Position::new_raw(pos).expect("linear_to_angle must be provided a valid position")
        },
    };

    settings.perform();
    drop(settings); // Stop after finished.
}
