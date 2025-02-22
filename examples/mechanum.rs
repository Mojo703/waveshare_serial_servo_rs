use serialport::SerialPort;
use waveshare_serial_servo::{
    hardware::ID,
    servo::{Servo, ServoError},
};

#[path = "./common/lib.rs"]
mod common;

struct Body {
    front_left: Servo,
    front_right: Servo,
    back_left: Servo,
    back_right: Servo,
}

impl Body {
    fn ping_all(&self, port: &mut Box<dyn SerialPort>) -> Result<(), ServoError> {
        self.front_left.ping(port)?;
        self.front_right.ping(port)?;
        self.back_left.ping(port)?;
        self.back_right.ping(port)?;

        Ok(())
    }
}

fn main() {
    let mut port = common::get_port();

    let body = Body {
        front_left: Servo::new(ID::single(1).unwrap()),
        front_right: Servo::new(ID::single(2).unwrap()),
        back_left: Servo::new(ID::single(3).unwrap()),
        back_right: Servo::new(ID::single(4).unwrap()),
    };

    body.ping_all(&mut port)
        .expect("Body must be set up with valid servos.");
}
