use std::{
    io::{stdin, stdout, Write},
    str::FromStr,
    time::Duration,
};

use serialport::{SerialPort, SerialPortInfo};
use waveshare_serial_servo::hardware::ID;

pub fn get_port() -> Box<dyn SerialPort> {
    // Print port options
    let avaliable = serialport::available_ports().expect("There must be avaliable serial ports.");
    for (index, port) in avaliable.iter().enumerate() {
        println!("port {index}: {port:?}");
    }
    println!("Choose a port:");
    let index = ask::<usize, _>(|index| avaliable.get(*index).is_some());
    let port: &SerialPortInfo = avaliable.get(index).expect("Chosen port must exist.");

    let baud_rate = 1_000_000;

    println!("Found a valid port: {:?}", port.port_name);

    serialport::new(port.port_name.clone(), baud_rate)
        .timeout(Duration::from_micros(10))
        .open()
        .expect("Serial port must open.")
}

pub fn ask<T: FromStr, F: Fn(&T) -> bool>(is_valid: F) -> T {
    loop {
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .expect("stdin read_line must work.");
        let Ok(value) = input.trim().parse() else {
            println!("Invalid value.");
            continue;
        };
        if !is_valid(&value) {
            println!("Invalid value.");
            continue;
        }
        return value;
    }
}

#[allow(unused)]
pub fn ask_id() -> ID {
    loop {
        print!("ID: ");
        let _ = stdout().flush();
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .expect("stdin read_line must work.");
        let Ok(value) = input.trim().parse::<u8>() else {
            println!("Invalid: not a unsigned 8bit integer.");
            continue;
        };
        match ID::try_from(value) {
            Err(e) => {
                println!("Invalid: not a valid id. Error: {e}");
                continue;
            }
            Ok(id) => return id,
        }
    }
}
