#[macro_use]
extern crate log;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::rmt::config::TransmitConfig;
use esp_idf_hal::rmt::{FixedLengthSignal, PinState, Pulse, TxRmtDriver};
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

    let p = Peripherals::take().expect("No Peripherals!");

    let pin = p.pins.gpio8;
    let channel = p.rmt.channel0;
    let config = TransmitConfig::new().clock_divider(1);
    let mut tx = TxRmtDriver::new(channel, pin, &config).expect("Error getting remote util");

    let mut hue = 0;

    loop {
        let rgb = hsb_to_rgb(hue, 255, 255);

        neopixel(rgb, &mut tx).expect("Error setting led");

        std::thread::sleep(Duration::from_millis(20));

        hue = (hue + 1) % 1536;
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

fn hsb_to_rgb(hue: u16, saturation: u8, brightness: u8) -> RGB {
    if saturation == 0 {
        RGB {
            r: brightness,
            g: brightness,
            b: brightness,
        }
    } else {
        let hue = hue % 1536;

        let offset = match hue {
            0..=255 => hue,
            256..=511 => hue - 256,
            512..=767 => hue - 512,
            768..=1023 => hue - 768,
            1024..=1279 => hue - 1024,
            1280..=1535 => hue - 1280,
            _ => unreachable!(),
        } as u8;

        let off = scale8(brightness, 255 - saturation);
        let fade_out = scale8(brightness, 255 - scale8(saturation, offset));
        let fade_in = scale8(brightness, 255 - scale8(saturation, 255 - offset));

        match hue {
            0..=255 => RGB {
                r: brightness,
                g: fade_in,
                b: off,
            },
            256..=511 => RGB {
                r: fade_out,
                g: brightness,
                b: off,
            },
            512..=767 => RGB {
                r: off,
                g: brightness,
                b: fade_in,
            },
            768..=1023 => RGB {
                r: off,
                g: fade_out,
                b: brightness,
            },
            1024..=1279 => RGB {
                r: fade_in,
                g: off,
                b: brightness,
            },
            1280..=1535 => RGB {
                r: brightness,
                g: off,
                b: fade_out,
            },
            _ => unreachable!(),
        }
    }
}

fn scale8(a: u8, b: u8) -> u8 {
    ((a as u16 * b as u16) >> 8) as u8
}
