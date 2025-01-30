# Waveshare Serial Servo

A 3rd party library to control waveshare's serial servos.

Developed with:

- Windows.
- [Bus Servo Adapter (A)](https://www.waveshare.com/catalog/product/view/id/5832/s/bus-servo-adapter-a) connected via USB.
- [ST3020 Servo](https://www.waveshare.com/product/st3020-servo.htm).
- 3S LiPo Battery (11.1V).

## Examples

The examples will automatically detect the Bus Servo Adapter (Based on "CH343" in the port product description). If this doesn't work, you may need to change the `common::is_valid_port` function.

Each example can be run with a command like `cargo run --example ping`.
