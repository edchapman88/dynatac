mod epd;
mod epdisplay;

use epdisplay::{Colour, DisplayError};
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::AnyInputPin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::{SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;

use std::thread;

const WIDTH: usize = 240;
const HEIGHT: usize = 320;
const BUFFER_SIZE: usize = WIDTH * HEIGHT / 8;

fn main() -> anyhow::Result<()> {
    // initialize runtime + logging
    link_patches();
    EspLogger::initialize_default();
    log::info!("start");

    // Take peripherals once and leak (so we can move references into threads)
    let peripherals = Box::leak(Box::new(
        esp_idf_hal::peripherals::Peripherals::take().unwrap(),
    ));

    // Spawn a thread that owns all SPI + display work.
    // The 'move' closure captures the leaked 'peripherals' reference (which is 'static).
    let builder = thread::Builder::new().stack_size(32 * 1024);
    let handle = builder.spawn(move || {
        // build SPI + pins inside task context
        let spi = &mut peripherals.spi2;
        let sclk = &mut peripherals.pins.gpio36; // SCK (board wiring)
        let sdo = &mut peripherals.pins.gpio33; // MOSI
        let sdi: Option<AnyInputPin> = None; // MISO if needed
        let cs = &mut peripherals.pins.gpio34; // CS

        let driver_config = SpiDriverConfig::new();
        let spi_driver = SpiDriver::new(spi, sclk, sdo, sdi, &driver_config).unwrap();

        let spi_device_config = SpiConfig::new().baudrate(115200.Hz());
        let spi_device_driver =
            SpiDeviceDriver::new(spi_driver, Some(cs), &spi_device_config).unwrap();

        log::info!("SPI initialized successfully in display task");

        // control pins (on your board)
        let busy = PinDriver::input(&mut peripherals.pins.gpio37).unwrap(); // BUSY
        let dc = PinDriver::output(&mut peripherals.pins.gpio35).unwrap(); // DC
        let mut led_en = PinDriver::output(&mut peripherals.pins.gpio42).unwrap();
        led_en.set_high().ok();

        let delay = Ets;

        // Create the display instance (owned by this task)
        let mut display = epd::Epd310Gdeq031t10::new(spi_device_driver, dc, busy, delay);

        // small logger adapter
        fn logger(s: &str) {
            log::info!("{s}")
        }

        log::info!("about to init display in thread");
        if let Err(e) = display.init() {
            log::error!("display init error: {e:?}");
            return;
        }
        log::info!("done init display");

        display.set_rotation(1);
        display.set_full_window();
        display.first_page();
        display.fill_screen(0x00);
        // display.draw();

        match display.next_page(logger) {
            Ok(_) => log::info!("display sequence finished"),
            Err(e) => log::error!("display sequence error: {e:?}"),
        }

        // keep the task alive so we can inspect logs / avoid dropping peripherals immediately
        loop {
            // sleep inside task - this doesn't block other tasks
            esp_idf_hal::delay::FreeRtos::delay_ms(10_000);
        }
    });

    // Option A: join the thread (blocks here) - optional
    // handle.join().expect("display thread panicked");

    // Option B: let main continue (we'll block here indefinitely)
    loop {
        esp_idf_hal::delay::FreeRtos::delay_ms(60_000);
    }

    // Ok(())  -- unreachable due to loop above
}
