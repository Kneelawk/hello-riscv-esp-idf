#[macro_use]
extern crate log;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::rmt::{FixedLengthSignal, PinState, Pulse, TxRmtDriver};
#[allow(unused)]
use esp_idf_sys as _;
use std::time::Duration;
use esp_idf_hal::rmt::config::TransmitConfig;
use rand::{Rng, thread_rng};

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

    let p = Peripherals::take().expect("No Peripherals!");

    let pin = p.pins.gpio8;
    let channel = p.rmt.channel0;
    let config = TransmitConfig::new().clock_divider(1);
    let mut tx = TxRmtDriver::new(channel, pin, &config).expect("Error getting remote util");

    let mut rand = thread_rng();

    loop {
        neopixel(RGB {
            r: rand.gen(),
            g: rand.gen(),
            b: rand.gen(),
        }, &mut tx).expect("Error setting led");

        std::thread::sleep(Duration::from_millis(100));

        neopixel(RGB {
            r: 0,
            g: 0,
            b: 0,
        }, &mut tx).expect("Error setting led");

        info!("Blinked.");

        std::thread::sleep(Duration::from_millis(100));
    }
}

// copied from neopixel example

struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

fn ns(nanos: u64) -> Duration {
    Duration::from_nanos(nanos)
}

fn neopixel(rgb: RGB, tx: &mut TxRmtDriver) -> anyhow::Result<()> {
    // e.g. rgb: (1,2,4)
    // G        R        B
    // 7      0 7      0 7      0
    // 00000010 00000001 00000100
    let color: u32 = ((rgb.g as u32) << 16) | ((rgb.r as u32) << 8) | rgb.b as u32;
    let ticks_hz = tx.counter_clock()?;
    let t0h = Pulse::new_with_duration(ticks_hz, PinState::High, &ns(350))?;
    let t0l = Pulse::new_with_duration(ticks_hz, PinState::Low, &ns(800))?;
    let t1h = Pulse::new_with_duration(ticks_hz, PinState::High, &ns(700))?;
    let t1l = Pulse::new_with_duration(ticks_hz, PinState::Low, &ns(600))?;
    let mut signal = FixedLengthSignal::<24>::new();
    for i in (0..24).rev() {
        let p = 2_u32.pow(i);
        let bit = p & color != 0;
        let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
        signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }
    tx.start_blocking(&signal)?;

    Ok(())
}
