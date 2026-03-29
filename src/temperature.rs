use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn read_trimmed_file(path: &Path) -> io::Result<String> {
    fs::read_to_string(path).map(|contents| contents.trim().to_lowercase())
}

fn is_cpu_sensor(path: &Path, cpu_sensor_keywords: &[String]) -> io::Result<bool> {
    let candidate_paths = [path.join("name"), path.join("device").join("label")];

    for candidate_path in candidate_paths {
        match read_trimmed_file(&candidate_path) {
            Ok(contents) => {
                if cpu_sensor_keywords
                    .iter()
                    .any(|keyword| contents.contains(keyword))
                {
                    return Ok(true);
                }
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(io::Error::new(
                    err.kind(),
                    format!(
                        "Failed to read sensor metadata '{}': {err}",
                        candidate_path.display()
                    ),
                ));
            }
        }
    }

    Ok(false)
}

fn read_temperature_celsius(path: PathBuf) -> io::Result<f32> {
    let contents = read_trimmed_file(&path).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!(
                "Failed to read temperature file '{}': {err}",
                path.display()
            ),
        )
    })?;
    let raw = contents.parse::<i64>().map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Failed to parse temperature from '{}': {err}",
                path.display()
            ),
        )
    })?;

    Ok(raw as f32 / 1000.0)
}

/// Reads the CPU temperature from /sys/class/hwmon by looking for sensors
/// whose name or label contains one of the configured keywords.
pub fn get_cpu_temp(cpu_sensor_keywords: &[String]) -> io::Result<f32> {
    let hwmon_root = Path::new("/sys/class/hwmon");
    let entries = fs::read_dir(hwmon_root).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("Failed to read '{}': {err}", hwmon_root.display()),
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("Failed to read hwmon directory entry: {err}"),
            )
        })?;
        let path = entry.path();

        if is_cpu_sensor(&path, cpu_sensor_keywords)? {
            let temp_input_path = path.join("temp1_input");
            return read_temperature_celsius(temp_input_path);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Could not find a CPU temperature sensor under /sys/class/hwmon",
    ))
}
