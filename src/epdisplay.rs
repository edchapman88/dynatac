// UC8253 Driver datasheet: https://v4.cecdn.yun300.cn/100001_1909185148/UC8253.pdf

use std::fmt::Debug;

const WIDTH: usize = 240;
const HEIGHT: usize = 320;
// In bytes (1bit per pixel)
pub const BUFFER_SIZE: usize = WIDTH * HEIGHT / 8;

const POWER_OFF: u8 = 0x02;
const PANEL_SETTING: u8 = 0x00;
const POWER_SETTING: u8 = 0x01;
const POWER_ON: u8 = 0x04;
const BOOSTER_SOFT_START: u8 = 0x06;
const DISPLAY_REFRESH: u8 = 0x12;
const DATA_START_TRANSMISSION_1: u8 = 0x10;
const DATA_START_TRANSMISSION_2: u8 = 0x13;
const VCOM_AND_DATA_INTERVAL_SETTING: u8 = 0x50;
const RESOLUTION_SETTING: u8 = 0x61;
const BUFFER_INIT_BYTE: u8 = 0x00; // black

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour {
    BLACK = 0x00,
    WHITE = 0xFF,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetBuf {
    Previous = DATA_START_TRANSMISSION_1,
    Next = DATA_START_TRANSMISSION_2,
}

struct DisplayDriver<SPI, DC, BUSY> {
    spi: SPI,
    dc: DC,
    busy: BUSY,
}

pub struct EPDisplay<SPI, DC, BUSY> {
    driver: DisplayDriver<SPI, DC, BUSY>,
    buf: Box<[u8; BUFFER_SIZE]>,
}

impl<SPI, DC, BUSY> EPDisplay<SPI, DC, BUSY> {
    pub fn new(spi: SPI, dc: DC, busy: BUSY) -> EPDisplay<SPI, DC, BUSY> {
        Self {
            driver: DisplayDriver { spi, dc, busy },
            buf: Box::new([BUFFER_INIT_BYTE; BUFFER_SIZE]),
        }
    }
}

#[derive(Debug)]
pub enum DisplayError {
    General(String),
}
impl DisplayError {
    pub fn from_debug<T: Debug>(e: T) -> Self {
        DisplayError::General(format!("{:?}", e))
    }
}

impl<SPI, DC, BUSY> DisplayDriver<SPI, DC, BUSY>
where
    SPI: embedded_hal::spi::SpiDevice,
    DC: embedded_hal::digital::OutputPin,
    BUSY: embedded_hal::digital::InputPin,
{
    fn write_command(&mut self, cmd: u8) -> Result<(), DisplayError> {
        self.dc.set_low().map_err(DisplayError::from_debug)?; // command mode
        self.spi.write(&[cmd]).map_err(DisplayError::from_debug)?;
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), DisplayError> {
        self.dc.set_high().map_err(DisplayError::from_debug)?; // data mode
        self.spi.write(data).map_err(DisplayError::from_debug)?;
        Ok(())
    }

    fn busy_wait(&mut self) {
        log::info!("entered busy");
        while self.busy.is_high().unwrap_or(false) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::info!("exit busy");
    }
    fn reset(&mut self) -> Result<(), DisplayError> {
        log::info!("entered reset");
        std::thread::sleep(std::time::Duration::from_millis(2));
        log::info!("finished reset");
        Ok(())
    }
    fn init(&mut self) -> Result<(), DisplayError> {
        // self.reset()?;

        // self.write_command(POWER_SETTING)?;
        // self.write_data(&[0x03, 0x10, 0x3F, 0x3F, 0x0D])?;

        // self.write_command(BOOSTER_SOFT_START)?;
        // self.write_data(&[0x17, 0x17, 0x17])?;

        self.write_command(PANEL_SETTING)?;
        self.write_data(&[0x1E, 0x0D])?;
        esp_idf_hal::delay::FreeRtos::delay_ms(10);
        self.write_command(PANEL_SETTING)?;
        self.write_data(&[0x1F, 0x0D])?;

        // self.write_command(RESOLUTION_SETTING)?;
        // self.write_data(&[0x1E, 0x01, 0x40])?;

        Ok(())
    }
}

impl<SPI, DC, BUSY> EPDisplay<SPI, DC, BUSY>
where
    SPI: embedded_hal::spi::SpiDevice,
    DC: embedded_hal::digital::OutputPin,
    BUSY: embedded_hal::digital::InputPin,
{
    pub fn init(&mut self) -> Result<(), DisplayError> {
        self.driver.init()?;
        Ok(())
    }

    pub fn full_refresh(&mut self) -> Result<(), DisplayError> {
        self.driver.init()?;
        self.driver.write_command(VCOM_AND_DATA_INTERVAL_SETTING)?;
        self.driver.write_data(&[0x97])?;
        self.driver.write_command(POWER_ON)?;
        self.driver.busy_wait();
        self.driver.write_command(DISPLAY_REFRESH)?;
        self.driver.busy_wait();
        Ok(())
    }
    pub fn write_fill(&mut self, target: TargetBuf, val: u8) -> Result<(), DisplayError> {
        self.driver.write_command(target as u8)?;
        self.driver.write_data(&*Box::new([val; BUFFER_SIZE]))?;
        Ok(())
    }
    pub fn clear(&mut self, colour: Colour) -> Result<(), DisplayError> {
        self.driver.init()?;
        self.write_fill(TargetBuf::Previous, colour as u8)?;
        self.write_fill(TargetBuf::Next, colour as u8)?;
        self.full_refresh()?;
        Ok(())
    }
}
