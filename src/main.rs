#![warn(clippy::all, rust_2018_idioms)]
#![allow(deprecated)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#[macro_use]
extern crate log;
extern crate factory_management_utils;
use factory_management_utils::utils;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    utils::log::setup_logger().expect("Logger couldn't be initialized");
    debug!("Sample debug");
    info!("running");
    warn!("sample warning");
    error!("Error message");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Factory Management Utils",
        native_options,
        Box::new(|cc| Box::new(factory_management_utils::FactoryManagementApp::new(cc))),
    );
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(factory_management_utils::FactoryManagementApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
