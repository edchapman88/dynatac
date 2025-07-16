// UC8253 Driver datasheet: https://v4.cecdn.yun300.cn/100001_1909185148/UC8253.pdf

use std::fmt::Debug;

const WIDTH: usize = 240;
const HEIGHT: usize = 320;
pub const BUFFER_SIZE: usize = WIDTH * HEIGHT / 8; // 1bpp

enum WRCycle {
    Read,
    Write,
}

enum DCType {
    Data,
    Command,
}

struct DisplayMsg {
    cycle: WRCycle,
    dc: DCType,
    data: u8,
}
impl DisplayMsg {
    const fn new(cycle: WRCycle, dc: DCType, data: u8) -> Self {
        DisplayMsg { cycle, dc, data }
    }
}

const POWER_OFF: u8 = 0x02;
const PANEL_SETTING: DisplayMsg = DisplayMsg::new(WRCycle::Write, DCType::Command, 0x00);
const POWER_SETTING: DisplayMsg = DisplayMsg::new(WRCycle::Write, DCType::Command, 0x01);
const POWER_ON: DisplayMsg = DisplayMsg::new(WRCycle::Write, DCType::Command, 0x04);
const BOOSTER_SOFT_START: DisplayMsg = DisplayMsg::new(WRCycle::Write, DCType::Command, 0x06);
const DISPLAY_REFRESH: DisplayMsg = DisplayMsg::new(WRCycle::Write, DCType::Command, 0x12);
const DATA_START_TRANSMISSION_1: DisplayMsg =
    DisplayMsg::new(WRCycle::Write, DCType::Command, 0x10);
const DATA_START_TRANSMISSION_2: DisplayMsg =
    DisplayMsg::new(WRCycle::Write, DCType::Command, 0x13);
const VCOM_AND_DATA_INTERVAL_SETTING: DisplayMsg =
    DisplayMsg::new(WRCycle::Write, DCType::Command, 0x50);

pub struct EPDisplay<SPI, DC, BUSY> {
    spi: SPI,
    dc: DC,
    busy: BUSY,
}
impl<SPI, DC, BUSY> EPDisplay<SPI, DC, BUSY> {
    pub fn new(spi: SPI, dc: DC, busy: BUSY) -> EPDisplay<SPI, DC, BUSY> {
        Self { spi, dc, busy }
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

impl<SPI, DC, BUSY> EPDisplay<SPI, DC, BUSY>
where
    SPI: embedded_hal::spi::SpiDevice,
    DC: embedded_hal::digital::OutputPin,
    BUSY: embedded_hal::digital::InputPin,
{
    pub fn write_command(&mut self, cmd: u8) -> Result<(), DisplayError> {
        self.dc.set_low().map_err(DisplayError::from_debug)?; // command mode
        self.spi.write(&[cmd]).map_err(DisplayError::from_debug)?;
        Ok(())
    }

    pub fn write_data(&mut self, data: &[u8]) -> Result<(), DisplayError> {
        self.dc.set_high().map_err(DisplayError::from_debug)?; // data mode
        self.spi.write(data).map_err(DisplayError::from_debug)?;
        Ok(())
    }

    pub fn busy_wait(&mut self) {
        log::info!("entered busy");
        while self.busy.is_high().unwrap_or(false) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::info!("exit busy");
    }
    pub fn reset(&mut self) -> Result<(), DisplayError> {
        log::info!("entered reset");

        // self.write_command(POWER_OFF)?;
        std::thread::sleep(std::time::Duration::from_millis(2));
        // self.write_command(POWER_ON.data)?;
        // self.busy_wait(); // Wait until not busy

        // Simulate power-on reset delay
        // std::thread::sleep(std::time::Duration::from_millis(10));
        log::info!("finished reset");
        Ok(())
    }
    pub fn init(&mut self) -> Result<(), DisplayError> {
        self.reset()?;

        // log::info!("test busy");
        // self.busy_wait();

        self.write_command(POWER_SETTING.data)?;
        self.write_data(&[0x03, 0x10, 0x3F, 0x3F, 0x0D])?;

        self.write_command(BOOSTER_SOFT_START.data)?;
        self.write_data(&[0x17, 0x17, 0x17])?;

        self.write_command(PANEL_SETTING.data)?;
        self.write_data(&[0x9E, 0x8D])?; // LUT from OTP
                                         // self.write_data(&[0x0E, 0x8D])?; // LUT from OTP

        self.write_command(POWER_ON.data)?;
        self.busy_wait();

        self.write_command(VCOM_AND_DATA_INTERVAL_SETTING.data)?;
        self.write_data(&[0xD7])?;

        self.write_command(DATA_START_TRANSMISSION_1.data)?;
        self.write_data(&[0x00; BUFFER_SIZE])?;
        // esp_idf_hal::delay::FreeRtos::delay_ms(100);
        self.write_command(DATA_START_TRANSMISSION_2.data)?;
        self.write_data(&[0x00; BUFFER_SIZE])?;
        self.write_command(DISPLAY_REFRESH.data)?;
        self.busy_wait();

        Ok(())
    }
    pub fn display_frame(&mut self, buf: &[u8]) -> Result<(), DisplayError> {
        self.write_command(DATA_START_TRANSMISSION_1.data)?;
        self.write_data(buf)?;

        self.write_command(DISPLAY_REFRESH.data)?;
        self.busy_wait();

        Ok(())
    }
}
