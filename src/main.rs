mod config;
mod display;
mod temperature;

use config::AppConfig;
use display::write_to_cpu_fan_display;
use hidapi::HidApi;
use log::{info, warn};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = AppConfig::load()?;
    let api = HidApi::new()?;
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let signal_flag = Arc::clone(&shutdown_requested);

    ctrlc::set_handler(move || {
        signal_flag.store(true, Ordering::Relaxed);
    })?;

    let device = api
        .open(config.vendor_id, config.product_id)
        .map_err(|e| format!("Failed to open HID device: {}", e))?;

    info!(
        "Connected to HID device {:#06x} using config {}",
        config.product_id,
        config.source_path.display()
    );

    while !shutdown_requested.load(Ordering::Relaxed) {
        if let Err(err) = write_to_cpu_fan_display(&device, &config) {
            warn!("{err}");
        }

        sleep_until_next_cycle(&shutdown_requested, config.update_interval);
    }

    info!("Shutdown signal received, exiting");
    Ok(())
}

fn sleep_until_next_cycle(shutdown_requested: &AtomicBool, interval: Duration) {
    const SLEEP_SLICE: Duration = Duration::from_millis(200);

    let mut remaining = interval;
    while !shutdown_requested.load(Ordering::Relaxed) && remaining > Duration::ZERO {
        let current_sleep = remaining.min(SLEEP_SLICE);
        thread::sleep(current_sleep);
        remaining = remaining.saturating_sub(current_sleep);
    }
}
