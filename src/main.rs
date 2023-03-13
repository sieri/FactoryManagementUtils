#![warn(clippy::all, rust_2018_idioms)]
#![allow(deprecated)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use factory_management_utils::utils;
#[cfg(target_arch = "wasm32")]
use log::error;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    utils::log::setup_logger().expect("Logger couldn't be initialized");

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
        let res = eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(factory_management_utils::FactoryManagementApp::new(cc))),
        )
        .await;
        if let Err(e) = res {
            error!("Eframe can't be loaded, probably need webGL{:?}", e)
        }
    });
}
