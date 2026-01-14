#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use freya::prelude::*;

mod app;

use app::init;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(droid_app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(init)
                .with_size(500., 450.)
                .with_window_attributes(|_attr, event_loop_builder| {
                    event_loop_builder.with_android_app(droid_app)
                }),
        ),
    )
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(init).with_size(600., 450.)))
}
