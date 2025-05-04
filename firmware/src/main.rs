#![no_std]
#![no_main]

use defmt::{info, trace};
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime };
use embassy_stm32::rcc::mux::Adcsel;
use embassy_time::Delay;
use embedded_hal_1::delay::DelayNs;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // dfmt trace
    trace!("Starting up...");
    let mut peripheral_config = embassy_stm32::Config::default();
    {
        peripheral_config.rcc.mux.adcsel = Adcsel::PER;
    }

    let p = embassy_stm32::init(peripheral_config);

    let mut delay = Delay;

    let mut adc = Adc::new(p.ADC3);

    adc.set_sample_time(SampleTime::CYCLES810_5);
    let mut vrefint = adc.enable_vrefint();
    let mut temp = adc.enable_temperature();

    delay.delay_us(50);

    for _ in 0..8 {
        adc.blocking_read(&mut vrefint);
    }


    let vrefint_sample = adc.blocking_read(&mut vrefint);
    let convert_to_millivolts = |sample: u16| {
        // From http://www.st.com/resource/en/datasheet/DM00071990.pdf
        // 6.3.24 Reference voltage
        const VREFINT_MV: u32 = 1216; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample))
    };

    let convert_to_celcius = |sample| {
        // From http://www.st.com/resource/en/datasheet/DM00071990.pdf
        // 6.3.22 Temperature sensor characteristics
        const V30: i32 = 620; // mV
        const AVG_SLOPE: f32 = 2.0; // mV/C

        let sample_mv = convert_to_millivolts(sample) as i32;

        let ts_cal1 = unsafe { *(0x1FF1_E820 as *const u16) } as i32; // @30 °C
        let ts_cal2 = unsafe { *(0x1FF1_E840 as *const u16) } as i32; // @130 °C
        (sample as i32 - ts_cal1) as f32 * (130.0 - 30.0) / (ts_cal2 - ts_cal1) as f32 + 30.0
    };

    info!("VrefInt: {}", vrefint_sample);
    const MAX_ADC_SAMPLE: u16 = (1 << 12) - 1;
    info!("VCCA: {} mV", convert_to_millivolts(MAX_ADC_SAMPLE));

    loop {
        delay.delay_ms(1000);

        let v = adc.blocking_read(&mut temp);
        info!("PC1: {} ({} mV)", v, convert_to_millivolts(v));

        let v = adc.blocking_read(&mut temp);
        let celcius = convert_to_celcius(v);
        info!("Internal temp: {} ({} C)", v, celcius);

        let v = adc.blocking_read(&mut vrefint);
        info!("VrefInt: {}", v);
        info!("Internal temp: {} ({} C)", v, celcius);
    }
}
