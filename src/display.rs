use crate::config::AppConfig;
use crate::temperature::get_cpu_temp;
use hidapi::HidDevice;
use std::io;

pub fn write_to_cpu_fan_display(device: &HidDevice, config: &AppConfig) -> io::Result<usize> {
    let temp = get_cpu_temp(&config.cpu_sensor_keywords)?;
    let temp_int = temp.round().clamp(0.0, u8::MAX as f32) as u8;
    let buf = [0, temp_int];

    device
        .write(&buf)
        .map_err(|err| io::Error::other(format!("Failed to write HID command: {err}")))
}
