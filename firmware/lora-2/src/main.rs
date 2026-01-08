#![no_std]
#![no_main]

// LoRaWAN interface variant module for RF switch control
mod iv;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Pin, Speed},
    i2c::{Config as I2cConfig, EventInterruptHandler, ErrorInterruptHandler, I2c},
    peripherals::{self, I2C2, PA11, PA12},
    rcc::*,
    rng::{self, Rng},
    spi::Spi,
    time::Hertz,
    Config,
};
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use {defmt_rtt as _, panic_probe as _};

// LoRaWAN imports (not used yet, but prepared)
use lora_phy::lorawan_radio::LorawanRadio;
use lora_phy::sx126x::{self, Stm32wl, Sx126x, TcxoCtrlVoltage};
use lora_phy::LoRa;
use lorawan_device::async_device::{region, Device, EmbassyTimer, JoinMode, JoinResponse};
use lorawan_device::default_crypto::DefaultFactory;
use lorawan_device::region::{Subband, AU915};
use lorawan_device::{AppEui, AppKey, DevEui};

use self::iv::{InterruptHandler, Stm32wlInterfaceVariant, SubghzSpiDevice};

bind_interrupts!(struct I2c2Irqs {
    I2C2_EV => EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => ErrorInterruptHandler<peripherals::I2C2>;
});

bind_interrupts!(struct Irqs{
    SUBGHZ_RADIO => InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

// SHT41 sensor constants
const SHT41_ADDR: u8 = 0x44;
const CMD_MEASURE_HIGH_PRECISION: u8 = 0xFD;

// LoRaWAN configuration constants
const MAX_TX_POWER: u8 = 14; // AU915 max TX power

// LoRaWAN credentials (from gateway TOT application)
// Note: EUIs are stored in LITTLE-ENDIAN for over-the-air transmission
const DEV_EUI: [u8; 8] = [0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x23]; // 23ce1bfeff091fac reversed
const APP_EUI: [u8; 8] = [0x56, 0x53, 0x29, 0xC5, 0x64, 0xA8, 0x30, 0xB1]; // b130a864c5295356 reversed
const APP_KEY: [u8; 16] = [
    0xB7, 0x26, 0x73, 0x9B, 0x78, 0xEC, 0x4B, 0x9E,
    0x92, 0x34, 0xE5, 0xD3, 0x5E, 0xA9, 0x68, 0x1B,
]; // AppKey stays in big-endian (MSB first)

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("====================================");
    info!("  STM32WL55 LoRa-1 - SHT41");
    info!("  Temperature & Humidity Sensor + LoRaWAN");
    info!("====================================");

    // Clock configuration matching working solution (HSE + PLL for radio stability)
    let mut config = Config::default();
    {
        config.rcc.hse = Some(Hse {
            freq: Hertz(32_000_000),
            mode: HseMode::Bypass,
            prescaler: HsePrescaler::DIV1,
        });
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV2,
            mul: PllMul::MUL6,
            divp: None,
            divq: Some(PllQDiv::DIV2),
            divr: Some(PllRDiv::DIV2),
        });
    }
    let p = embassy_stm32::init(config);

    info!("STM32WL55 initialized with HSE + PLL clock");

    // Test I2C2 bus and detect devices
    {
        info!("Testing I2C2: PA12 (SCL), PA11 (SDA)");
        let mut i2c_config = I2cConfig::default();
        i2c_config.sda_pullup = true;
        i2c_config.scl_pullup = true;

        // SAFETY: This is the first and only time we're using these peripherals
        let mut i2c = unsafe {
            I2c::new_blocking(
                I2C2::steal(),
                PA12::steal(),
                PA11::steal(),
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

    // ============================================
    // Initialize LoRaWAN Radio Hardware
    // ============================================
    info!("Initializing LoRaWAN radio hardware...");

    // RF switch control pins (NUCLEO-WL55JC1 board)
    let ctrl1 = Output::new(p.PC4.degrade(), Level::Low, Speed::High);
    let ctrl2 = Output::new(p.PC5.degrade(), Level::Low, Speed::High);
    let ctrl3 = Output::new(p.PC3.degrade(), Level::High, Speed::High);
    info!("✓ RF switch pins configured (PC3, PC4, PC5)");

    // Initialize SubGHz SPI
    let spi = Spi::new_subghz(p.SUBGHZSPI, p.DMA1_CH1, p.DMA1_CH2);
    let spi = SubghzSpiDevice(spi);
    info!("✓ SubGHz SPI initialized");

    // Configure radio
    let use_high_power_pa = true; // Use high power PA for better range
    let config = sx126x::Config {
        chip: Stm32wl { use_high_power_pa },
        tcxo_ctrl: Some(TcxoCtrlVoltage::Ctrl1V7),
        use_dcdc: true,
        rx_boost: false,
    };

    // Create interface variant with RF switch control
    let iv = Stm32wlInterfaceVariant::new(
        Irqs,
        use_high_power_pa,
        Some(ctrl1),
        Some(ctrl2),
        Some(ctrl3),
    )
    .unwrap();
    info!("✓ RF switch interface variant created");

    // Initialize LoRa radio (this will be used for LoRaWAN later)
    let lora = LoRa::new(Sx126x::new(spi, iv, config), true, Delay)
        .await
        .unwrap();
    info!("✓ LoRa radio initialized");

    // Convert to LorawanRadio wrapper
    let radio: LorawanRadio<_, _, MAX_TX_POWER> = lora.into();

    // Configure AU915 region
    let mut au915 = AU915::new();
    au915.set_join_bias(Subband::_1); // Sub-band 1 (915.2-916.6 MHz channels 0-7)
    let region: region::Configuration = au915.into();
    info!("✓ AU915 region configured (sub-band 1)");

    // Initialize RNG for crypto
    let rng = Rng::new(p.RNG, Irqs);
    info!("✓ RNG initialized for crypto");

    // Create LoRaWAN device
    let mut device: Device<_, DefaultFactory, _, _> =
        Device::new(region, radio, EmbassyTimer::new(), rng);
    info!("✓ LoRaWAN device created");

    info!("========================================");
    info!("  LoRaWAN stack initialized!");
    info!("  Ready to join network");
    info!("========================================");

    // ============================================
    // OTAA Join with Retry Logic
    // ============================================
    info!("Starting OTAA join procedure...");
    info!("DevEUI: {:02X}", DEV_EUI);
    info!("AppEUI: {:02X}", APP_EUI);

    let join_mode = JoinMode::OTAA {
        deveui: DevEui::from(DEV_EUI),
        appeui: AppEui::from(APP_EUI),
        appkey: AppKey::from(APP_KEY),
    };

    let mut join_attempt = 1;
    const MAX_JOIN_ATTEMPTS: u8 = 5;

    loop {
        info!("OTAA join attempt {}/{}", join_attempt, MAX_JOIN_ATTEMPTS);

        match device.join(&join_mode).await {
            Ok(JoinResponse::JoinSuccess) => {
                info!("✓ LoRaWAN network joined successfully!");
                break;
            }
            Ok(JoinResponse::NoJoinAccept) => {
                error!("✗ Join failed (attempt {}): No join accept received", join_attempt);

                if join_attempt >= MAX_JOIN_ATTEMPTS {
                    error!("Maximum join attempts reached. Stopping.");
                    loop {
                        // Infinite loop - device cannot proceed without network connection
                        Timer::after_secs(60).await;
                    }
                }

                join_attempt += 1;
                info!("Waiting 10 seconds before retry...");
                Timer::after_secs(10).await;
            }
            Err(err) => {
                error!("✗ Join error (attempt {}): {:?}", join_attempt, err);

                if join_attempt >= MAX_JOIN_ATTEMPTS {
                    error!("Maximum join attempts reached. Stopping.");
                    loop {
                        // Infinite loop - device cannot proceed without network connection
                        Timer::after_secs(60).await;
                    }
                }

                join_attempt += 1;
                info!("Waiting 10 seconds before retry...");
                Timer::after_secs(10).await;
            }
        }
    }

    info!("========================================");
    info!("  LoRaWAN OTAA Join Complete!");
    info!("  Device is now connected");
    info!("========================================");

    // Initialize LED
    let mut led = Output::new(p.PB15, Level::Low, Speed::Low);

    // Text style
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    info!("Starting sensor + display loop...");

    let mut temp_int = 0i16;  // Integer temperature (Celsius)
    let mut hum_int = 0i16;   // Integer humidity (% RH)
    let mut uplink_counter = 0u32;  // Track loop iterations for uplink timing
    let mut tx_count = 0u32;  // Track number of uplinks sent
    const UPLINK_INTERVAL: u32 = 30; // Send uplink every 30 loops (30 * 2s = 60s)

    loop {
        uplink_counter += 1;
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
            I2c::new_blocking(
                I2C2::steal(),
                PA12::steal(),
                PA11::steal(),
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
        // Step 2: Update OLED display (SSD1306 128x32)
        // ============================================
        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        if display.init().is_ok() {
            // Get current SNR and RSSI from device
            let snr = device.last_snr();
            let rssi = device.last_rssi();

            // Clear display
            let _ = display.clear(BinaryColor::Off);

            // Line 1: Title + TX count (128x32 = 4 lines max at 6x10 font)
            let mut line1 = heapless::String::<32>::new();
            let _ = core::fmt::write(&mut line1, format_args!("LoRa-1  TX:{}", tx_count));
            let _ = Text::new(&line1, Point::new(0, 6), text_style)
                .draw(&mut display);

            // Line 2: Temperature and Humidity
            let mut line2 = heapless::String::<32>::new();
            let _ = core::fmt::write(&mut line2, format_args!("{}C {}%", temp_int, hum_int));
            let _ = Text::new(&line2, Point::new(0, 16), text_style)
                .draw(&mut display);

            // Line 3: SNR and RSSI (only show if we have valid data, rssi != 0)
            if rssi != 0 {
                let mut line3 = heapless::String::<32>::new();
                let _ = core::fmt::write(&mut line3, format_args!("S:{} R:{}", snr, rssi));
                let _ = Text::new(&line3, Point::new(0, 26), text_style)
                    .draw(&mut display);
            } else {
                // Before first uplink, show "Joined"
                let _ = Text::new("Joined", Point::new(0, 26), text_style)
                    .draw(&mut display);
            }

            // Flush to display
            let _ = display.flush();
        } else {
            error!("✗ Failed to init OLED");
        }

        // display and i2c are dropped here, releasing the hardware

        // ============================================
        // Step 3: Send LoRaWAN uplink every 60 seconds
        // ============================================
        if uplink_counter >= UPLINK_INTERVAL {
            uplink_counter = 0;

            // Encode payload: 4 bytes total
            // Bytes 0-1: Temperature (°C * 100) as signed 16-bit big-endian
            // Bytes 2-3: Humidity (% * 100) as unsigned 16-bit big-endian
            let temp_encoded = (temp_int * 100) as i16;
            let hum_encoded = (hum_int * 100) as u16;

            let payload: [u8; 4] = [
                (temp_encoded >> 8) as u8,  // Temp MSB
                temp_encoded as u8,          // Temp LSB
                (hum_encoded >> 8) as u8,    // Humidity MSB
                hum_encoded as u8,           // Humidity LSB
            ];

            info!("Sending uplink: temp={}.{}°C, hum={}%", temp_int, temp_encoded.abs() % 100, hum_int);

            // Send unconfirmed uplink on FPort 1
            match device.send(&payload, 1, false).await {
                Ok(response) => {
                    tx_count += 1;
                    let snr = device.last_snr();
                    let rssi = device.last_rssi();
                    info!("✓ Uplink sent successfully: {:?}", response);
                    info!("  SNR: {} dB, RSSI: {} dBm, TX count: {}", snr, rssi, tx_count);
                }
                Err(err) => {
                    error!("✗ Uplink failed: {:?}", err);
                }
            }

            // Wait a bit after TX to allow radio to settle
            Timer::after_millis(100).await;
        }

        Timer::after_secs(2).await;
    }
}
