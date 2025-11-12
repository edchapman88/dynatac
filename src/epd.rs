use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::{Operation, SpiDevice};

use crate::epdisplay::Colour;

const WIDTH: u16 = 240;
const HEIGHT: u16 = 320;
const BUFFER_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize) / 8;

const FULL_REFRESH_TIME_MS: u32 = 1100;
const POWER_ON_TIME_MS: u32 = 50;
const POWER_OFF_TIME_MS: u32 = 50;
const PARTIAL_REFRESH_TIME_MS: u32 = 700;

pub struct Epd310Gdeq031t10<SPI, DC, BUSY, DELAY> {
    spi: SPI,
    dc: DC,
    busy: BUSY,
    delay: DELAY,
    rotation: u8,
    power_is_on: bool,
    init_display_done: bool,
    initial_refresh: bool,
    initial_write: bool,
    using_partial_mode: bool,
    partial_dimensions: (i16, i16, i16, i16),
    pub buffer: [u8; BUFFER_SIZE],
}

impl<SPI, DC, BUSY, DELAY> Epd310Gdeq031t10<SPI, DC, BUSY, DELAY>
where
    SPI: SpiDevice,
    DC: OutputPin,
    BUSY: embedded_hal::digital::InputPin,
    DELAY: DelayNs,
{
    pub fn new(spi: SPI, dc: DC, busy: BUSY, delay: DELAY) -> Self {
        Self {
            spi,
            dc,
            busy,
            delay,
            rotation: 0,
            power_is_on: false,
            init_display_done: false,
            initial_refresh: true,
            initial_write: true,
            using_partial_mode: false,
            partial_dimensions: (0, 0, WIDTH as i16, HEIGHT as i16),
            buffer: [0xFFu8; BUFFER_SIZE],
        }
    }
    pub fn init(&mut self) -> Result<(), SPI::Error> {
        // Panel Setting (soft reset)
        self.write_command(0x00)?;
        self.write_data(&[0x1e, 0x0d])?;
        self.delay.delay_ms(1);

        self.power_is_on = false;

        // Panel Setting (main)
        self.write_command(0x00)?;
        self.write_data(&[0x1f, 0x0d])?;
        self.init_display_done = true;

        // Power On
        // self.write_command(0x04)?;
        // self.wait_while_busy(50);

        Ok(())
    }

    pub fn update_full(&mut self) -> Result<(), SPI::Error> {
        self.write_command(0xE0)?; // Cascade Setting (CCSET)
        self.write_data(&[0x02 as u8])?; // TSFIX
        self.write_command(0xE5)?; // Force Temperature (TSSET)
        self.write_data(&[0x5A as u8])?; // 90, 1015000us
        self.write_command(0x50)?;
        self.write_data(&[0x97 as u8])?;
        self.power_on()?;
        self.write_command(0x12)?; //display refresh
        self.wait_while_busy(FULL_REFRESH_TIME_MS);
        self.init_display_done = false; // needed, reason unknown
        Ok(())
    }

    pub fn update_part(&mut self) -> Result<(), SPI::Error> {
        self.write_command(0xE0)?; // Cascade Setting (CCSET)
        self.write_data(&[0x02 as u8])?; // TSFIX
        self.write_command(0xE5)?; // Force Temperature (TSSET)
        self.write_data(&[0x79 as u8])?; // 121
        self.write_command(0x50)?;
        self.write_data(&[0xD7 as u8])?;
        self.power_on()?;
        self.write_command(0x12)?;
        self.wait_while_busy(PARTIAL_REFRESH_TIME_MS);
        self.init_display_done = false;
        Ok(())
    }

    pub fn power_on(&mut self) -> Result<(), SPI::Error> {
        if !self.power_is_on {
            self.write_command(0x04)?;
            self.wait_while_busy(POWER_ON_TIME_MS);
        }
        self.power_is_on = true;
        Ok(())
    }

    pub fn power_off(&mut self) -> Result<(), SPI::Error> {
        if self.power_is_on {
            self.write_command(0x02)?;
            self.wait_while_busy(POWER_OFF_TIME_MS);
        }
        self.power_is_on = false;
        Ok(())
    }

    pub fn refresh_full(&mut self) -> Result<(), SPI::Error> {
        self.update_full()?;
        self.initial_refresh = false;
        Ok(())
    }

    pub fn refresh_part(&mut self, x: i16, y: i16, w: i16, h: i16) -> Result<(), SPI::Error> {
        if self.initial_refresh {
            self.refresh_full()
        } else {
            // intersection with screen
            let mut w1 = if x < 0 { w + x } else { w }; // reduce
            let mut h1 = if y < 0 { h + y } else { h }; // reduce
            let mut x1 = if x < 0 { 0 } else { x }; // limit
            let y1 = if y < 0 { 0 } else { y }; // limit
            w1 = if x1 + w1 < WIDTH as i16 {
                w1
            } else {
                WIDTH as i16 - x1
            }; // limit
            h1 = if y1 + h1 < HEIGHT as i16 {
                h1
            } else {
                HEIGHT as i16 - y1
            }; // limit
            if (w1 <= 0) || (h1 <= 0) {
                return Ok(());
            };
            // make x1, w1 multiple of 8
            w1 += x1 % 8;
            if w1 % 8 > 0 {
                w1 += 8 - w1 % 8
            };
            x1 -= x1 % 8;
            self.write_command(0x91)?; // partial in
            self.set_partial_ram_area(x1 as u16, y1 as u16, w1 as u16, h1 as u16)?;
            self.update_part()?;
            self.write_command(0x92) // partial out
        }
    }
    pub fn set_partial_ram_area(
        &mut self,
        mut x: u16,
        y: u16,
        w: u16,
        h: u16,
    ) -> Result<(), SPI::Error> {
        let xe = (x + w - 1) | 0x0007; // byte boundary inclusive (last byte)
        let ye = y + h - 1;
        x &= 0xFFF8; // byte boundary
        self.write_command(0x90)?;
        self.write_data(&x.to_ne_bytes())?;
        self.write_data(&xe.to_ne_bytes())?;
        self.write_data(&(y / 256).to_ne_bytes())?;
        self.write_data(&(y % 256).to_ne_bytes())?;
        self.write_data(&(ye / 256).to_ne_bytes())?;
        self.write_data(&(ye % 256).to_ne_bytes())?;
        self.write_data(&[0x01 as u8])
    }

    fn _write_screen_buffer(&mut self, command: u8, value: u8) -> Result<(), SPI::Error> {
        if !self.init_display_done {
            self.init()?;
        };
        self.write_command(command)?;
        self.transfer(&[value; (WIDTH as usize * HEIGHT as usize / 8)])
    }

    pub fn clear_screen(&mut self, value: u8) -> Result<(), SPI::Error> {
        self._write_screen_buffer(0x10, value)?;
        self._write_screen_buffer(0x13, value)?;
        self.refresh_full()?;
        self.initial_refresh = false;
        Ok(())
    }

    pub fn write_screen_buffer(&mut self, value: u8) -> Result<(), SPI::Error> {
        if self.initial_write {
            self.clear_screen(value)
        } else {
            self._write_screen_buffer(0x13, value)
        }
    }

    pub fn write_screen_buffer_again(&mut self, value: u8) -> Result<(), SPI::Error> {
        self._write_screen_buffer(0x10, value)
    }

    pub fn write_image(
        &mut self,
        bitmap: &[u8],
        x: i16,
        y: i16,
        w: i16,
        h: i16,
        invert: bool,
        mirror_y: bool,
    ) -> Result<(), SPI::Error> {
        self._write_image(0x13, bitmap, x, y, w, h, invert, mirror_y)
    }

    pub fn write_image_again(
        &mut self,
        bitmap: &[u8],
        x: i16,
        y: i16,
        w: i16,
        h: i16,
        invert: bool,
        mirror_y: bool,
    ) -> Result<(), SPI::Error> {
        self._write_image(0x10, bitmap, x, y, w, h, invert, mirror_y)
    }

    pub fn write_image_for_full_refresh(
        &mut self,
        bitmap: &[u8],
        x: i16,
        y: i16,
        w: i16,
        h: i16,
        invert: bool,
        mirror_y: bool,
    ) -> Result<(), SPI::Error> {
        self._write_image(0x10, bitmap, x, y, w, h, invert, mirror_y)?;
        self._write_image(0x13, bitmap, x, y, w, h, invert, mirror_y)
    }
    fn _write_image(
        &mut self,
        command: u8,
        bitmap: &[u8],
        mut x: i16,
        y: i16,
        mut w: i16,
        h: i16,
        invert: bool,
        mirror_y: bool,
    ) -> Result<(), SPI::Error> {
        self.delay.delay_ms(1);
        let wb = (w + 7) / 8; // width bytes, bitmaps are padded
        x -= x % 8; // byte boundary
        w = wb * 8; // byte boundary
        let x1 = if x < 0 { 0 } else { x }; // limit
        let y1 = if y < 0 { 0 } else { y }; // limit
        let mut w1 = if x + w < WIDTH as i16 {
            w
        } else {
            WIDTH as i16 - x
        }; // limit
        let mut h1 = if y + h < HEIGHT as i16 {
            h
        } else {
            HEIGHT as i16 - y
        }; // limit
        let dx = x1 - x;
        let dy = y1 - y;
        w1 -= dx;
        h1 -= dy;
        if (w1 <= 0) || (h1 <= 0) {
            return Ok(());
        };
        if !self.init_display_done {
            self.init()?;
        };
        if self.initial_write {
            self.write_screen_buffer(0xFF)?
        };
        self.write_command(0x91)?;
        self.set_partial_ram_area(x1 as u16, y1 as u16, w1 as u16, h1 as u16)?;
        self.write_command(command)?;

        let bytes_per_row = (w1 / 8) as usize;

        let mut out = Vec::with_capacity((bytes_per_row * (h1 as usize)) as usize);
        for i in 0..h1 {
            for j in 0..(w1 / 8) {
                let idx = if mirror_y {
                    j + dx / 8 + (h - 1 - (i + dy)) * wb
                } else {
                    j + dx / 8 + (i + dy) * wb
                };
                let mut data = bitmap[idx as usize];
                if invert {
                    data = !data
                };
                out.push(data);
            }
        }

        const CHUNK: usize = 1024;
        for chunk in out.chunks(CHUNK) {
            self.transfer(chunk)?;
            esp_idf_hal::delay::FreeRtos::delay_ms(1); // yield to feed watchdog
        }

        self.write_command(0x92)?;
        self.delay.delay_ms(1);
        Ok(())
    }

    fn write_command(&mut self, command: u8) -> Result<(), SPI::Error> {
        self.dc.set_low().ok();
        self.spi.transaction(&mut [Operation::Write(&[command])])?;
        self.dc.set_high().ok();
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.dc.set_high().ok();
        self.spi.transaction(&mut [Operation::Write(data)])?;
        Ok(())
    }

    fn transfer(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.spi.transfer(&mut [], data)
    }

    fn wait_while_busy(&mut self, timeout_ms: u32) {
        let mut waited = 0;
        while self.busy.is_high().unwrap_or(false) && waited < timeout_ms {
            self.delay.delay_ms(1);
            waited += 1;
        }
    }

    pub fn set_rotation(&mut self, rot: u8) {
        self.rotation = rot % 4;
    }

    pub fn first_page(&mut self) {
        self.fill_screen(0xFF);
        // self.current_page = 0;
        // self.second_phase = false;
    }

    pub fn next_page(&mut self, logger: fn(&str) -> ()) -> Result<bool, SPI::Error> {
        let (x, y, w, h) = self.partial_dimensions;
        if self.using_partial_mode {
            logger("using partial mode");
            self.write_image(&self.buffer.clone(), x, y, w, h, false, false)?;
            self.refresh_part(x, y, w, h)?;
            self.write_image_again(&self.buffer.clone(), x, y, w, h, false, false)?;
        } else {
            logger("not partial mode");
            self.write_image_for_full_refresh(
                &self.buffer.clone(),
                0,
                0,
                WIDTH as i16,
                HEIGHT as i16,
                false,
                false,
            )?;
            logger("wrote screen for full refresh");
            self.refresh_full()?;
            logger("did refresh full");
            self.write_image_again(&self.buffer.clone(), x, y, w, h, false, false)?;
            logger("wrote image again");
            self.power_off()?;
            logger("powered off");
        }
        return Ok(false);
    }

    pub fn set_full_window(&mut self) {
        self.using_partial_mode = false;
        self.partial_dimensions = (0, 0, WIDTH as i16, HEIGHT as i16);
    }

    pub fn fill_screen(&mut self, val: u8) {
        self.buffer = [val; BUFFER_SIZE];
    }

    pub fn draw(&mut self) {
        self.buffer[50..4000].fill(Colour::BLACK as u8);
    }
}
