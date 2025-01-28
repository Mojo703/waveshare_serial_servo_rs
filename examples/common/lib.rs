use serialport::{SerialPort, SerialPortInfo};

/// Search for a CH343 USB to serial port.
fn is_valid_port(info: &serialport::UsbPortInfo) -> bool {
    info.product
        .as_ref()
        .is_some_and(|text| text.contains("CH343"))
}

pub fn get_port() -> Box<dyn SerialPort> {
    let port = serialport::available_ports()
        .expect("There must be avaliable serial ports.")
        .into_iter()
        .find(
            |SerialPortInfo {
                 port_name: _,
                 port_type,
             }| {
                match port_type {
                    serialport::SerialPortType::UsbPort(info) => is_valid_port(info),
                    _ => false,
                }
            },
        )
        .expect("The servo driver must be connected.");

    let baud_rate = 1_000_000;

    println!("Found a valid port: {:?}", port.port_name);

    serialport::new(port.port_name, baud_rate)
        .open()
        .expect("Serial port must open.")
}
