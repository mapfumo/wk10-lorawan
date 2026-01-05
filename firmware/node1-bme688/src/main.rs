#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    dma::NoDma,
    gpio::{Level, Output, Speed},
    i2c::{Config as I2cConfig, EventInterruptHandler, ErrorInterruptHandler, I2c},
    peripherals::{self, I2C2, PA11, PA12},
    time::Hertz,
    Config,
};
use embassy_time::Timer;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use sh1106::{prelude::*, Builder};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct I2c2Irqs {
    I2C2_EV => EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => ErrorInterruptHandler<peripherals::I2C2>;
});

const SHT41_ADDR: u8 = 0x44;
const CMD_MEASURE_HIGH_PRECISION: u8 = 0xFD;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("====================================");
    info!("  STM32WL55 - I2C2 OLED + SHT41");
    info!("  Node 1 - Temperature & Humidity");
    info!("====================================");

    let config = Config::default();
    let p = embassy_stm32::init(config);

    info!("STM32WL55 initialized");

    // Test I2C2 bus and detect devices
    {
        info!("Testing I2C2: PA12 (SCL), PA11 (SDA)");
        let mut i2c_config = I2cConfig::default();
        i2c_config.sda_pullup = true;
        i2c_config.scl_pullup = true;

        // SAFETY: This is the first and only time we're using these peripherals
        let mut i2c = unsafe {
            I2c::new(
                I2C2::steal(),
                PA12::steal(),
                PA11::steal(),
                I2c2Irqs,
                NoDma,
                NoDma,
                Hertz(100_000),
                i2c_config,
            )
        };

        // Try to wake up SHT41 first
        info!("Attempting to wake up SHT41 @ 0x{:02X}...", SHT41_ADDR);
        let wake_result = i2c.blocking_write(SHT41_ADDR, &[CMD_MEASURE_HIGH_PRECISION]);
        match wake_result {
            Ok(_) => info!("✓ SHT41 wake command sent successfully"),
            Err(_) => info!("✗ SHT41 wake failed"),
        }

        Timer::after_millis(100).await;

        // Scan I2C bus - full scan to find all devices
        info!("Scanning I2C2 bus (full scan)...");
        let mut found_count = 0;
        for addr in 0x00..=0x7F {
            let mut buf = [0u8; 1];
            if i2c.blocking_read(addr, &mut buf).is_ok() {
                info!("✓ Device at 0x{:02X}", addr);
                found_count += 1;
            }
        }
        info!("Total devices found: {}", found_count);
    }

    // Initialize LED
    let mut led = Output::new(p.PB15, Level::Low, Speed::Low);

    // Text style
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    info!("Starting sensor + display loop...");

    let mut temp_int = 0i16;  // Integer temperature (Celsius)
    let mut hum_int = 0i16;   // Integer humidity (% RH)

    loop {
        // Toggle LED
        led.toggle();

        // ============================================
        // Step 1: Create I2C and read SHT41 sensor
        // ============================================
        let mut i2c_config = I2cConfig::default();
        i2c_config.sda_pullup = true;
        i2c_config.scl_pullup = true;

        // SAFETY: We're creating a temporary I2C instance that will be dropped
        // at the end of each loop iteration, releasing the hardware for reuse
        let mut i2c = unsafe {
            I2c::new(
                I2C2::steal(),
                PA12::steal(),
                PA11::steal(),
                I2c2Irqs,
                NoDma,
                NoDma,
                Hertz(100_000),
                i2c_config,
            )
        };

        // Send measurement command
        if i2c.blocking_write(SHT41_ADDR, &[CMD_MEASURE_HIGH_PRECISION]).is_ok() {
            // Wait for measurement (typ 8.3ms for high precision)
            Timer::after_millis(10).await;

            // Read 6 bytes: temp_msb, temp_lsb, temp_crc, hum_msb, hum_lsb, hum_crc
            let mut data = [0u8; 6];
            if i2c.blocking_read(SHT41_ADDR, &mut data).is_ok() {
                // Convert raw values to temperature and humidity
                // Temperature: T = -45 + 175 * (raw / 65535)
                // Humidity: RH = -6 + 125 * (raw / 65535)

                let temp_raw = ((data[0] as u16) << 8) | (data[1] as u16);
                let hum_raw = ((data[3] as u16) << 8) | (data[4] as u16);

                // Use integer math: T = -45 + (175 * raw) / 65535
                temp_int = -45 + ((175 * temp_raw as i32) / 65535) as i16;
                hum_int = -6 + ((125 * hum_raw as i32) / 65535) as i16;

                info!("✓ SHT41: {}°C, {}% RH", temp_int, hum_int);
            } else {
                info!("✗ Failed to read SHT41 data");
            }
        } else {
            info!("✗ Failed to send SHT41 command");
        }

        // ============================================
        // Step 2: Update OLED display
        // ============================================
        let mut display: GraphicsMode<_> = Builder::new()
            .with_size(DisplaySize::Display128x64)
            .connect_i2c(i2c)
            .into();

        if display.init().is_ok() {
            // Clear display
            display.clear();

            // Title
            let _ = Text::new("STM32WL55 Node1", Point::new(5, 10), text_style)
                .draw(&mut display);

            // Temperature
            let mut temp_buf = heapless::String::<32>::new();
            let _ = core::fmt::write(&mut temp_buf, format_args!("Temp: {} C", temp_int));
            let _ = Text::new(&temp_buf, Point::new(5, 28), text_style)
                .draw(&mut display);

            // Humidity
            let mut hum_buf = heapless::String::<32>::new();
            let _ = core::fmt::write(&mut hum_buf, format_args!("Hum:  {} %", hum_int));
            let _ = Text::new(&hum_buf, Point::new(5, 43), text_style)
                .draw(&mut display);

            // Status
            let _ = Text::new("SHT41 Active", Point::new(20, 58), text_style)
                .draw(&mut display);

            // Flush to display
            let _ = display.flush();
        } else {
            error!("✗ Failed to init OLED");
        }

        // display and i2c are dropped here, releasing the hardware

        Timer::after_secs(2).await;
    }
}
