#![deny(unsafe_code)]
#![no_std]
#![no_main]

use airy as _;

use cortex_m_rt::entry;

use crate::hal::{dwt::DwtExt, i2c::I2c, prelude::*, stm32};
use core::fmt::Write;
use stm32f4xx_hal as hal;

use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use heapless::String;
use hm3301::Hm3301;
use ssd1306::{prelude::*, Builder as SSD1306Builder, I2CDIBuilder};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        defmt::debug!("Starting...");
        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        defmt::debug!("Setup led");
        let gpioc = dp.GPIOC.split();
        let mut led = gpioc.pc13.into_push_pull_output();

        let dwt = cp.DWT.constrain(cp.DCB, clocks);
        let mut delay = dwt.delay();

        led.set_high().unwrap();
        delay.delay_ms(100_u32);

        // Set up I2C - SCL is PB8 and SDA is PB9; they are set to Alternate Function 4
        // as per the STM32F411xC/E datasheet page 60.
        defmt::debug!("Setup I2C Pin 8 and 9");
        let gpiob = dp.GPIOB.split();
        let scl = gpiob.pb8.into_alternate_af4_open_drain();
        let sda = gpiob.pb9.into_alternate_af4_open_drain();
        let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks);

        defmt::debug!("Setup shared bus");
        let bus = shared_bus::BusManagerSimple::new(i2c);

        defmt::debug!("Setup HM3301 on I2c");
        let mut sensor = Hm3301::new(bus.acquire_i2c());
        sensor.enable_i2c().unwrap();

        led.set_low().unwrap();
        delay.delay_ms(100_u32);

        defmt::debug!("Setup SSD1306 Display on shared I2C");
        let interface = I2CDIBuilder::new().init(bus.acquire_i2c());
        let mut disp: GraphicsMode<_> = SSD1306Builder::new().connect(interface).into();
        disp.init().unwrap();
        disp.flush().unwrap();
        let text_style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .build();

        let mut lines: [String<heapless::consts::U32>; 4] =
            [String::new(), String::new(), String::new(), String::new()];
        let mut errors: u32 = 0;
        defmt::debug!("Starting loop");
        loop {
            delay.delay_ms(800_u32);

            led.set_low().unwrap();
            delay.delay_ms(100_u32);

            led.set_high().unwrap();
            delay.delay_ms(100_u32);

            if let Ok(m) = sensor.read_measurement() {
                for line in lines.iter_mut() {
                    line.clear();
                }
                write!(lines[0], "Airy Sensor!").unwrap();
                write!(lines[1], "PM2.5: atm {}", m.atm_pm25).unwrap();
                write!(lines[2], "PM1: atm {}", m.atm_pm1).unwrap();
                write!(lines[3], "PM10: atm {} Err: {}", m.atm_pm10, errors).unwrap();
                defmt::debug!("{:str}", lines[1].as_str());

                disp.clear();
                for (i, line) in lines.iter().enumerate() {
                    Text::new(line, Point::new(0, i as i32 * 16))
                        .into_styled(text_style)
                        .draw(&mut disp)
                        .unwrap_or_else(|_| errors += 1);
                }
                disp.flush().unwrap_or_else(|_| errors += 1);
            } else {
                errors += 1;
            }
        }
    }

    airy::exit();
}
