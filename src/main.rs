use embedded_hal::delay::DelayNs;
use esp_idf_hal::gpio::AnyInputPin;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::{SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;

use esp_idf_hal::{delay::Ets, gpio::PinDriver, prelude::*, spi};

mod display;
use display::DisplayError;

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    let peripherals = Box::leak(Box::new(
        esp_idf_hal::peripherals::Peripherals::take().unwrap(),
    ));

    // SPI2 is used for your external SPI devices, SPI0 or SPI1 are used
    // internally for memory and flash.
    let spi = &mut peripherals.spi2;
    let sclk = &mut peripherals.pins.gpio36; // Serial Clock
    let sdo = &mut peripherals.pins.gpio33; // Serial Data Out
    let sdi: Option<AnyInputPin> = None; // Serial Data In
    let cs = &mut peripherals.pins.gpio34; // Chip Select

    // SPI bus driver
    let driver_config = SpiDriverConfig::new();
    let spi_driver = SpiDriver::new(spi, sclk, sdo, sdi, &driver_config).unwrap();

    // SPI device
    let spi_device_config = SpiConfig::new().baudrate(115200.Hz());
    let spi_device_driver = SpiDeviceDriver::new(spi_driver, Some(cs), &spi_device_config).unwrap();

    log::info!("SPI initialized successfully!");

    // Pins wired up to the diplay controller.
    let busy = PinDriver::input(&mut peripherals.pins.gpio37)?; // Driver Busy
    let dc = PinDriver::output(&mut peripherals.pins.gpio35)?; // Data / Command control pin
                                                               //
    let mut led_en = PinDriver::output(&mut peripherals.pins.gpio42)?;
    led_en.set_high()?;

    let mut perif_en = PinDriver::output(&mut peripherals.pins.gpio41)?;
    // std::thread::sleep(std::time::Duration::from_secs(5));
    // perif_en.set_low()?;
    // log::info!("set perif power low");
    // std::thread::sleep(std::time::Duration::from_secs(5));
    // perif_en.set_high()?;
    // log::info!("set perif power high");
    //
    // led_en.set_low()?;

    // let mut delay = Ets;

    let mut dsp = display::EPDisplay::new(spi_device_driver, dc, busy);
    match dsp.init() {
        Err(DisplayError::General(e_str)) => log::error!("{}", e_str),
        Ok(()) => log::info!("display init success"),
    }
    // led_en.set_high()?;

    let buffer = [0xFF; display::BUFFER_SIZE / 2];
    dsp.display_frame(&buffer)
        .expect("failed writing to display");

    // esp_idf_hal::delay::FreeRtos::delay_ms(5000);
    // let buffer = [0x33; display::BUFFER_SIZE];
    // dsp.display_frame(&buffer)
    //     .expect("failed writing to display");

    // Construct the epaper driver, RST pin is None (-1 in Arduino)

    log::info!("Display updated successfully!");

    loop {
        esp_idf_hal::delay::FreeRtos::delay_ms(3000);
        // dsp.display_frame(&buffer)
        //     .expect("failed writing to display");
    }
}
