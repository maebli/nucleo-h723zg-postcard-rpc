#![no_std]
#![no_main]

use defmt::{info, println, trace};
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::rcc::mux::Adcsel;
use embassy_time::Delay;
use embedded_hal_1::delay::DelayNs;
use {defmt_rtt as _, panic_probe as _};

const VREFINT_MV: u32 = 1216; // mV DS13313 Rev 4
const T_SAMPLING_VREFINT_MIN_NS: u32 = 430; // us DS13313 Rev 4
const T_STARTUP_VREFINT_MAX_NS: u32 = 440; // us DS13313 Rev 4

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut peripheral_config = embassy_stm32::Config::default();
    {
        peripheral_config.rcc.mux.adcsel = Adcsel::PER;
    }

    let p = embassy_stm32::init(peripheral_config);

    let mut delay = Delay;
    let mut adc = Adc::new(p.ADC3);

    // adc clock = 32Mhz 
    let sample_time_us = 810.5/32.0; 
    assert!(sample_time_us >= T_SAMPLING_VREFINT_MIN_NS as f32 / 1000.0);

    adc.set_resolution(embassy_stm32::adc::Resolution::BITS12);
    adc.set_sample_time(SampleTime::CYCLES32_5);

    let mut vrefint_channel = adc.enable_vrefint();

    delay.delay_ns(T_STARTUP_VREFINT_MAX_NS);

    loop {
        let vrefint_sample = adc.blocking_read(&mut vrefint_channel);
        info!("VrefInt: {}", vrefint_sample);
        delay.delay_ms(1000);
    }
}
