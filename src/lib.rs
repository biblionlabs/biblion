pub mod app;
pub mod components;
pub mod dialog;
pub mod utils;

use freya::radio::RadioChannel;

pub const APP_NAME: &str = env!("CARGO_CRATE_NAME");

#[derive(Default, Clone)]
#[allow(dead_code)]
struct AppState {
    books: Vec<String>,
}

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
pub enum AppChannel {
    BooksSuggesions
}

impl RadioChannel<AppState> for AppChannel {}

#[cfg(target_os = "android")]
use {app::init, freya::prelude::*, winit::platform::android::activity::AndroidApp};

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
