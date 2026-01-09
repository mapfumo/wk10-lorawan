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
use sh1106::{prelude::*, Builder};
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

// BME680 sensor constants
const BME680_ADDR_PRIMARY: u8 = 0x76;   // SDO pin LOW or floating
const BME680_ADDR_SECONDARY: u8 = 0x77; // SDO pin HIGH

// BME680 register addresses
const BME680_REG_CHIP_ID: u8 = 0xD0;
const BME680_REG_CTRL_MEAS: u8 = 0x74;
const BME680_REG_CTRL_HUM: u8 = 0x72;
const BME680_REG_CTRL_GAS_1: u8 = 0x71;  // Gas control register
const BME680_REG_CTRL_GAS_0: u8 = 0x70;  // Heater control
const BME680_REG_GAS_WAIT_0: u8 = 0x64;  // Gas wait time
const BME680_REG_RES_HEAT_0: u8 = 0x5A;  // Heater resistance
const BME680_REG_PRESS_MSB: u8 = 0x1F;
const BME680_REG_TEMP_MSB: u8 = 0x22;
const BME680_REG_HUM_MSB: u8 = 0x25;
const BME680_REG_GAS_R_MSB: u8 = 0x2A;   // Gas resistance MSB
const BME680_REG_GAS_R_LSB: u8 = 0x2B;   // Gas resistance LSB + range

// BME680 control values
const BME680_OSRS_H_X2: u8 = 0x02;  // Humidity oversampling x2

// LoRaWAN configuration constants
const MAX_TX_POWER: u8 = 14; // AU915 max TX power

// LoRaWAN credentials (from gateway TOT application)
// Note: EUIs are stored in LITTLE-ENDIAN for over-the-air transmission
// LoRa-2 uses different DevEUI (24ce... instead of 23ce...) to avoid conflicts
const DEV_EUI: [u8; 8] = [0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x24]; // 24ce1bfeff091fac reversed
const APP_EUI: [u8; 8] = [0x56, 0x53, 0x29, 0xC5, 0x64, 0xA8, 0x30, 0xB1]; // b130a864c5295356 reversed
const APP_KEY: [u8; 16] = [
    0xB7, 0x26, 0x73, 0x9B, 0x78, 0xEC, 0x4B, 0x9E,
    0x92, 0x34, 0xE5, 0xD3, 0x5E, 0xA9, 0x68, 0x1B,
]; // AppKey stays in big-endian (MSB first)

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("====================================");
    info!("  STM32WL55 LoRa-2 - BME688");
    info!("  Environmental Sensor + LoRaWAN");
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

        // Try to read BME688 chip ID at both addresses
        info!("Attempting to read BME680 chip ID...");
        let mut chip_id = [0u8; 1];

        // Try primary address (0x76)
        if i2c.blocking_write(BME680_ADDR_PRIMARY, &[BME680_REG_CHIP_ID]).is_ok()
            && i2c.blocking_read(BME680_ADDR_PRIMARY, &mut chip_id).is_ok() {
            info!("✓ BME680 found at 0x76 (primary), chip ID: 0x{:02X}", chip_id[0]);
        } else if i2c.blocking_write(BME680_ADDR_SECONDARY, &[BME680_REG_CHIP_ID]).is_ok()
            && i2c.blocking_read(BME680_ADDR_SECONDARY, &mut chip_id).is_ok() {
            info!("✓ BME680 found at 0x77 (secondary), chip ID: 0x{:02X}", chip_id[0]);
        } else {
            info!("✗ BME680 not responding at 0x76 or 0x77");
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

    let mut temp_int = 0i16;     // Integer temperature (Celsius)
    let mut hum_int = 0i16;      // Integer humidity (% RH)
    let mut pressure_int = 0i16; // Integer pressure (hPa)
    let mut gas_int = 0u16;      // Gas resistance (kOhm) - not implemented yet
    let mut uplink_counter = 0u32;  // Track loop iterations for uplink timing
    let mut tx_count = 0u32;     // Track number of uplinks sent
    let mut snr = 0i8;           // Last SNR (dB)
    let mut rssi = 0i16;         // Last RSSI (dBm)
    const UPLINK_INTERVAL: u32 = 30; // Send uplink every 30 loops (30 * 2s = 60s)

    loop {
        uplink_counter += 1;
        // Toggle LED
        led.toggle();

        // ============================================
        // Step 1: Create I2C and read BME688 sensor
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

        // Scan I2C bus to see what's connected
        info!("Scanning I2C2 bus...");
        let mut found_devices = 0;
        for addr in 0x00..=0x7F {
            let mut buf = [0u8; 1];
            if i2c.blocking_read(addr, &mut buf).is_ok() {
                info!("  ✓ Device found at 0x{:02X}", addr);
                found_devices += 1;
            }
        }
        info!("Total I2C devices found: {}", found_devices);

        // Read BME680 sensor
        let mut sensor_found = false;
        let mut bme_addr = BME680_ADDR_PRIMARY; // Start with primary address (0x76)

        // Try primary address first, then secondary
        if i2c.blocking_write(bme_addr, &[BME680_REG_CTRL_HUM, BME680_OSRS_H_X2]).is_err() {
            bme_addr = BME680_ADDR_SECONDARY;
        }

        if i2c.blocking_write(bme_addr, &[BME680_REG_CTRL_HUM, BME680_OSRS_H_X2]).is_ok() {
            // Set sleep mode first (required)
            if i2c.blocking_write(bme_addr, &[BME680_REG_CTRL_MEAS, 0x00]).is_ok() {
                Timer::after_millis(20).await;

                // Configure gas heater for measurement
                // res_heat_0: heater resistance target (~300°C, value ~0x73 typical)
                let _ = i2c.blocking_write(bme_addr, &[BME680_REG_RES_HEAT_0, 0x73]);
                // gas_wait_0: heater duration (0x59 = 100ms with multiplier)
                let _ = i2c.blocking_write(bme_addr, &[BME680_REG_GAS_WAIT_0, 0x59]);
                // ctrl_gas_1: run_gas=1, nb_conv=0 (use heater profile 0)
                let _ = i2c.blocking_write(bme_addr, &[BME680_REG_CTRL_GAS_1, 0x10]);

                // Trigger forced measurement with gas: 0x25 = temp x1, press x1, forced mode
                if i2c.blocking_write(bme_addr, &[BME680_REG_CTRL_MEAS, 0x25]).is_ok() {
                    sensor_found = true;
                    // Wait for measurement + gas heater to complete
                    Timer::after_millis(2000).await;

                // Read temperature (3 bytes starting at 0x22)
                let mut temp_data = [0u8; 3];
                if i2c.blocking_write(bme_addr, &[BME680_REG_TEMP_MSB]).is_ok()
                    && i2c.blocking_read(bme_addr, &mut temp_data).is_ok() {

                    let temp_raw = ((temp_data[0] as u32) << 12)
                                 | ((temp_data[1] as u32) << 4)
                                 | ((temp_data[2] as u32) >> 4);

                    // Convert raw ADC to temperature (recalibrated: temp_raw ≈ 514000 → 28°C)
                    temp_int = (temp_raw / 18357) as i16;
                }

                // Read humidity (2 bytes starting at 0x25)
                let mut hum_data = [0u8; 2];
                if i2c.blocking_write(bme_addr, &[BME680_REG_HUM_MSB]).is_ok()
                    && i2c.blocking_read(bme_addr, &mut hum_data).is_ok() {

                    let hum_raw = ((hum_data[0] as u16) << 8) | (hum_data[1] as u16);

                    // Convert raw ADC to humidity (recalibrated: hum_raw ≈ 23180 → 60%)
                    let hum_tenths = ((hum_raw as i32 * 10) / 386) as i16;
                    hum_int = hum_tenths / 10;
                    if hum_int > 100 { hum_int = 100; }
                    if hum_int < 0 { hum_int = 0; }
                }

                // Read pressure (3 bytes starting at 0x1F)
                let mut press_data = [0u8; 3];
                if i2c.blocking_write(bme_addr, &[BME680_REG_PRESS_MSB]).is_ok()
                    && i2c.blocking_read(bme_addr, &mut press_data).is_ok() {

                    let press_raw = ((press_data[0] as u32) << 12)
                                  | ((press_data[1] as u32) << 4)
                                  | ((press_data[2] as u32) >> 4);

                    // Convert raw ADC to pressure (calibrated experimentally)
                    let press_pa = ((press_raw * 295) / 1000) as u32;
                    pressure_int = (press_pa / 100) as i16; // Convert Pa to hPa
                }

                // Read gas resistance (2 bytes at 0x2A-0x2B)
                // gas_r_msb[7:0] = ADC bits [9:2]
                // gas_r_lsb[7:6] = ADC bits [1:0], [5:4] = gas_valid + heat_stab, [3:0] = gas_range
                let mut gas_data = [0u8; 2];
                if i2c.blocking_write(bme_addr, &[BME680_REG_GAS_R_MSB]).is_ok()
                    && i2c.blocking_read(bme_addr, &mut gas_data).is_ok() {

                    let gas_adc = ((gas_data[0] as u16) << 2) | ((gas_data[1] as u16) >> 6);
                    let gas_range = gas_data[1] & 0x0F;
                    let gas_valid = (gas_data[1] >> 5) & 0x01;
                    let heat_stab = (gas_data[1] >> 4) & 0x01;

                    if gas_valid == 1 && heat_stab == 1 && gas_adc > 0 {
                        // Lookup table for gas range (from BME680 datasheet)
                        // These are the const_array1 values for resistance calculation
                        const GAS_RANGE_R1: [u32; 16] = [
                            2147483647, 2147483647, 2147483647, 2147483647,
                            2147483647, 2126008810, 2147483647, 2130303777,
                            2147483647, 2147483647, 2143188679, 2136746228,
                            2147483647, 2126008810, 2147483647, 2147483647,
                        ];
                        const GAS_RANGE_R2: [u32; 16] = [
                            4096000000, 2048000000, 1024000000, 512000000,
                            255744255, 127110228, 64000000, 32258064,
                            16016016, 8000000, 4000000, 2000000,
                            1000000, 500000, 250000, 125000,
                        ];

                        // Simplified resistance calculation (avoiding float)
                        // gas_res = (range_r2 / gas_adc) * range_factor
                        let range_idx = gas_range as usize;
                        if range_idx < 16 {
                            let var1 = GAS_RANGE_R2[range_idx] / (gas_adc as u32);
                            // Convert to kOhm (divide by 1000)
                            gas_int = (var1 / 1000) as u16;
                        }
                    }
                }

                info!("✓ BME680: {}°C, {}% RH, {} hPa, {} kOhm", temp_int, hum_int, pressure_int, gas_int);
                }
            }
        }

        if !sensor_found {
            info!("✗ BME680 not responding at 0x{:02X} or 0x{:02X}", BME680_ADDR_PRIMARY, BME680_ADDR_SECONDARY);
        }

        // ============================================
        // Step 2: Update OLED display (SH1106 128x64)
        // ============================================
        // 128x64 with 6x10 font = 21 chars x 6 lines
        let mut display: GraphicsMode<_> = Builder::new()
            .with_size(DisplaySize::Display128x64)
            .connect_i2c(i2c)
            .into();

        if display.init().is_ok() {
            Timer::after_millis(50).await;
            display.clear();
            Timer::after_millis(20).await;

            // Line 1: Title + TX count
            let mut line1 = heapless::String::<32>::new();
            let _ = core::fmt::write(&mut line1, format_args!("LoRa-2      Tx:{}", tx_count));
            let _ = Text::new(&line1, Point::new(0, 10), text_style).draw(&mut display);

            // Line 2: Temperature and Humidity
            let mut line2 = heapless::String::<32>::new();
            let _ = core::fmt::write(&mut line2, format_args!("Temp: {}C  Hum: {}%", temp_int, hum_int));
            let _ = Text::new(&line2, Point::new(0, 22), text_style).draw(&mut display);

            // Line 3: Pressure
            let mut line3 = heapless::String::<32>::new();
            let _ = core::fmt::write(&mut line3, format_args!("Press: {} hPa", pressure_int));
            let _ = Text::new(&line3, Point::new(0, 34), text_style).draw(&mut display);

            // Line 4: Gas resistance (future)
            let mut line4 = heapless::String::<32>::new();
            let _ = core::fmt::write(&mut line4, format_args!("Gas: {} kOhm", gas_int));
            let _ = Text::new(&line4, Point::new(0, 46), text_style).draw(&mut display);

            // Line 5: SNR and RSSI
            if rssi != 0 {
                let mut line5 = heapless::String::<32>::new();
                let _ = core::fmt::write(&mut line5, format_args!("SNR:{} RSSI:{}", snr, rssi));
                let _ = Text::new(&line5, Point::new(0, 58), text_style).draw(&mut display);
            } else {
                let _ = Text::new("Joined - awaiting TX", Point::new(0, 58), text_style).draw(&mut display);
            }

            let _ = display.flush();
        }

        // Display is automatically dropped here, releasing I2C

        // ============================================
        // Step 3: Send LoRaWAN uplink every 60 seconds
        // ============================================
        if uplink_counter >= UPLINK_INTERVAL {
            uplink_counter = 0;

            // Encode payload: 12 bytes total
            // Bytes 0-1: Temperature (°C * 100) as signed 16-bit big-endian
            // Bytes 2-3: Humidity (% * 100) as unsigned 16-bit big-endian
            // Bytes 4-5: Pressure (hPa * 10) as unsigned 16-bit big-endian
            // Bytes 6-7: Gas resistance (kOhm) as unsigned 16-bit big-endian
            // Bytes 8-11: Reserved (future use)
            let temp_encoded = (temp_int * 100) as i16;
            let hum_encoded = (hum_int * 100) as u16;
            let pressure_encoded = (pressure_int * 10) as u16;
            let gas_encoded = gas_int;

            let payload: [u8; 12] = [
                (temp_encoded >> 8) as u8,     // 0: Temp MSB
                temp_encoded as u8,             // 1: Temp LSB
                (hum_encoded >> 8) as u8,       // 2: Humidity MSB
                hum_encoded as u8,              // 3: Humidity LSB
                (pressure_encoded >> 8) as u8,  // 4: Pressure MSB
                pressure_encoded as u8,         // 5: Pressure LSB
                (gas_encoded >> 8) as u8,       // 6: Gas MSB
                gas_encoded as u8,              // 7: Gas LSB
                0, 0, 0, 0,                     // 8-11: Reserved
            ];

            info!("Sending uplink: {}°C, {}%, {} hPa, {} kOhm", temp_int, hum_int, pressure_int, gas_int);

            // Send unconfirmed uplink on FPort 1
            match device.send(&payload, 1, false).await {
                Ok(response) => {
                    tx_count += 1;
                    snr = device.last_snr() as i8;
                    rssi = device.last_rssi();
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
