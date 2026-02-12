#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use biblion::app::init;
use freya::prelude::*;

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::EnvFilter;

    use biblion::utils::data_dir;

    let builder = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_suffix("log")
        .build(data_dir(&["logs"]))
        .unwrap();
    let (non_blocking, _guard) = tracing_appender::non_blocking(builder);
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .try_from_env()
                .unwrap_or_default(),
        )
        .with_writer(non_blocking)
        .init();

    launch(LaunchConfig::new().with_window(WindowConfig::new(init).with_size(600., 450.)))
}
