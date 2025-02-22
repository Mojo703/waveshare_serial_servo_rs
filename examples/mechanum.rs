use waveshare_serial_servo::{hardware::ID, servo::Servo};

#[path = "./common/lib.rs"]
mod common;

struct Body {
    front_left: Servo,
    front_right: Servo,
    back_left: Servo,
    back_right: Servo,
}

fn main() {
    let mut port = common::get_port();

    let body = Body {
        front_left: Servo::new(ID::single(1).unwrap()),
        front_right: Servo::new(ID::single(2).unwrap()),
        back_left: Servo::new(ID::single(3).unwrap()),
        back_right: Servo::new(ID::single(4).unwrap()),
    };
}
