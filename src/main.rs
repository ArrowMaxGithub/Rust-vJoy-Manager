pub mod config;
pub mod error;
pub mod graphics_backend;
pub mod input;
pub mod manager;
pub mod previous;
pub mod rebind;
pub mod ui_data;

use egui::{epaint::Hsva, Color32};
use error::Error;
use log::{error, info};
use manager::Manager;

// use profiling::tracy_client;
use winit::{
    dpi::PhysicalSize,
    event_loop::{DeviceEventFilter, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

pub fn auto_color(i: usize) -> Color32 {
    Hsva::new(i as f32 * 0.618034, 0.85, 0.5, 1.0).into()
}

fn main() -> Result<(), Error> {
    init_logger();
    info!("Startup");
    #[cfg(feature = "profile")]
    {
        profiling::tracy_client::Client::start();
        profiling::register_thread!("Main Thread");
    }
    let (window, event_loop) = create_window("Rust vJoy Manager", [800, 600])?;
    let manager = Manager::new(&window, &event_loop)?;
    manager.run(window, event_loop)
}

fn create_window(title: &str, size: [u32; 2]) -> Result<(Window, EventLoop<()>), Error> {
    let event_loop = EventLoopBuilder::default().build();
    event_loop.set_device_event_filter(DeviceEventFilter::Never);
    let window = match WindowBuilder::new()
        .with_title(title)
        .with_inner_size(PhysicalSize::new(size[0], size[1]))
        .with_resizable(true)
        .build(&event_loop)
    {
        Ok(ok) => ok,
        Err(e) => return Err(Error::WindowCreateFailed { source: e }),
    };
    Ok((window, event_loop))
}

fn init_logger() {
    use std::env;
    use std::io::Write;
    env::set_var("RUST_BACKTRACE", "1");
    let env = env_logger::Env::default()
        .write_style_or("RUST_LOG_STYLE", "always")
        .filter_or("RUST_LOG", "trace");

    env_logger::Builder::from_env(env)
        .target(env_logger::Target::Stderr)
        .format(|buf, record| {
            let mut style = buf.style();

            match record.level() {
                log::Level::Info => style.set_color(env_logger::fmt::Color::Green),
                log::Level::Warn => style.set_color(env_logger::fmt::Color::Yellow),
                log::Level::Error => style.set_color(env_logger::fmt::Color::Red),
                _ => style.set_color(env_logger::fmt::Color::White),
            };

            let timestamp = buf.timestamp();

            writeln!(
                buf,
                "{:<20} : {:<5} : {}",
                timestamp,
                style.value(record.level()),
                record.args()
            )
        })
        .init();
    info!("Logger init for stderr");
}

/// Print error with source and exit.
pub(crate) fn print_error_and_exit(err: Box<dyn std::error::Error>) -> ! {
    error!("{}", err);
    if let Some(src) = err.source() {
        error!("Source: {}", src);
    }
    std::process::exit(1);
}
