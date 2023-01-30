use std::time::Duration;
#[allow(unused)]
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    println!("Hello, world!");

    // This crashes on Xtensa processors for some reason :/
    // Good thing this is a RISC-V processor :)
    std::thread::sleep(Duration::from_millis(1000));

    println!("Slept for 1 second!");
}
