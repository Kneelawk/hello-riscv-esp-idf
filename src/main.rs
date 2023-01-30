#[macro_use]
extern crate log;

#[allow(unused)]
use esp_idf_sys as _;
use std::time::Duration;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    // This crashes on Xtensa processors for some reason :/
    // Good thing this is a RISC-V processor :)
    std::thread::sleep(Duration::from_millis(1000));

    info!("Slept for 1 second!");
}
