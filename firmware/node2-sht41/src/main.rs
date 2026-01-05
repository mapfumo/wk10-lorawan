#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// LoRaWAN Device Credentials (to be filled from ChirpStack)
const DEV_EUI: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02]; // TODO: Update
const APP_EUI: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // TODO: Update
const APP_KEY: [u8; 16] = [0x00; 16]; // TODO: Update from ChirpStack

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("====================================");
    info!("  Node 2 - SHT41 LoRaWAN Sensor");
    info!("  STM32WL55JC1 - Week 10");
    info!("====================================");

    // Initialize STM32WL55 with default config
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux = ClockSrc::MSI(MsiRange::Range48mhz);
    }
    let _p = embassy_stm32::init(config);

    info!("STM32WL55 initialized");
    info!("DevEUI: {:02X}", DEV_EUI);
    info!("AppEUI: {:02X}", APP_EUI);

    // TODO: Initialize I2C1 for SHT41 and OLED (PB8/PB9)
    // TODO: Initialize SubGHz radio for LoRaWAN
    // TODO: Implement OTAA join procedure
    // TODO: Implement sensor reading loop
    // TODO: Implement uplink transmission

    info!("Entering main loop (placeholder)...");

    loop {
        info!("Heartbeat - waiting for implementation");
        Timer::after_secs(5).await;
    }
}
