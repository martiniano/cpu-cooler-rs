use serde::Deserialize;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

const APP_NAME: &str = "cpu_cooler";
const CONFIG_FILE_NAME: &str = "cpu_cooler.toml";
const STANDARD_CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_PATH_ENV_VAR: &str = "CPU_COOLER_CONFIG";

#[derive(Debug, Deserialize)]
struct RawConfig {
    vendor_id: String,
    product_id: String,
    update_interval_secs: u64,
    cpu_sensor_keywords: Vec<String>,
}

#[derive(Debug)]
pub struct AppConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    pub update_interval: Duration,
    pub cpu_sensor_keywords: Vec<String>,
    pub source_path: PathBuf,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = find_config_path()?;
        let contents = fs::read_to_string(&config_path)?;
        let raw: RawConfig = toml::from_str(&contents)?;
        let cpu_sensor_keywords: Vec<String> = raw
            .cpu_sensor_keywords
            .into_iter()
            .map(|keyword| keyword.trim().to_lowercase())
            .filter(|keyword| !keyword.is_empty())
            .collect();

        if cpu_sensor_keywords.is_empty() {
            return Err("cpu_sensor_keywords cannot be empty".into());
        }

        Ok(Self {
            vendor_id: parse_u16_field(&raw.vendor_id, "vendor_id")?,
            product_id: parse_u16_field(&raw.product_id, "product_id")?,
            update_interval: Duration::from_secs(raw.update_interval_secs),
            cpu_sensor_keywords,
            source_path: config_path,
        })
    }
}

fn find_config_path() -> io::Result<PathBuf> {
    if let Some(config_path) = std::env::var_os(CONFIG_PATH_ENV_VAR) {
        let config_path = PathBuf::from(config_path);
        if config_path.exists() {
            return Ok(config_path);
        }

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Environment variable {CONFIG_PATH_ENV_VAR} points to a missing file: {}",
                config_path.display()
            ),
        ));
    }

    for candidate in standard_config_paths() {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "Configuration file was not found. Set {CONFIG_PATH_ENV_VAR}, or place config at $XDG_CONFIG_HOME/{APP_NAME}/{STANDARD_CONFIG_FILE_NAME}, ~/.config/{APP_NAME}/{STANDARD_CONFIG_FILE_NAME}, /etc/{APP_NAME}/{STANDARD_CONFIG_FILE_NAME}, the current directory, or next to the executable"
        ),
    ))
}

fn standard_config_paths() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(xdg_config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        candidates.push(
            PathBuf::from(xdg_config_home)
                .join(APP_NAME)
                .join(STANDARD_CONFIG_FILE_NAME),
        );
    }

    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(
            PathBuf::from(home)
                .join(".config")
                .join(APP_NAME)
                .join(STANDARD_CONFIG_FILE_NAME),
        );
    }

    candidates.push(
        PathBuf::from("/etc")
            .join(APP_NAME)
            .join(STANDARD_CONFIG_FILE_NAME),
    );
    candidates.push(PathBuf::from(CONFIG_FILE_NAME));

    if let Ok(executable_path) = std::env::current_exe()
        && let Some(executable_dir) = executable_path.parent()
    {
        candidates.push(executable_dir.join(CONFIG_FILE_NAME));
    }

    candidates
}

fn parse_u16_field(value: &str, field_name: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let trimmed = value.trim();
    let parsed = if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16)
    } else {
        trimmed.parse::<u16>()
    };

    parsed.map_err(|err| format!("Invalid value for {field_name} ('{trimmed}'): {err}").into())
}
