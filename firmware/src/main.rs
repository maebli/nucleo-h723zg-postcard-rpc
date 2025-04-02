#![no_std]
#![no_main]

use defmt::{info, trace};
use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, peripherals::{USB_OTG_HS},usb};
use embassy_stm32::usb::{Driver, Instance,InterruptHandler};
use embassy_usb::{Config, UsbDevice};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use {defmt_rtt as _, embassy_stm32 as hal, panic_probe as _};
use static_cell::ConstStaticCell;
use embassy_time::{Duration, Timer};

use postcard_rpc::{
    define_dispatch,
    header::VarHeader,
    server::{
        impls::embassy_usb_v0_4::{
            dispatch_impl::{WireRxBuf, WireRxImpl, WireSpawnImpl, WireStorage, WireTxImpl},
            PacketBuffers,
        },
        Dispatch, Server,
    },
};
use icd::{PingEndpoint, ENDPOINT_LIST, TOPICS_IN_LIST, TOPICS_OUT_LIST};

pub struct Context;
type AppDriver = usb::Driver<'static, USB_OTG_HS>;
type AppStorage = WireStorage<ThreadModeRawMutex, AppDriver, 256, 256, 64, 256>;
type BufStorage = PacketBuffers<1024, 1024>;
type AppTx = WireTxImpl<ThreadModeRawMutex, AppDriver>;
type AppRx = WireRxImpl<AppDriver>;
type AppServer = Server<AppTx, AppRx, WireRxBuf, MyApp>;

static PBUFS: ConstStaticCell<BufStorage> = ConstStaticCell::new(BufStorage::new());
static STORAGE: AppStorage = AppStorage::new();

fn usb_config() -> Config<'static> {
    let mut config = Config::new(0x16c0, 0x27DD);
    config.manufacturer = Some("Nase");
    config.product = Some("flashy");
    config.serial_number = Some("12345678");

    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    config
}

bind_interrupts!(struct Irqs {
    OTG_HS => usb::InterruptHandler<USB_OTG_HS>;
});


define_dispatch! {
    app: MyApp;
    spawn_fn: spawn_fn;
    tx_impl: AppTx;
    spawn_impl: WireSpawnImpl;
    context: Context;

    endpoints: {
        list: ENDPOINT_LIST;

        | EndpointTy                | kind      | handler                       |
        | ----------                | ----      | -------                       |
        | PingEndpoint              | blocking  | ping_handler                  |
    };
    topics_in: {
        list: TOPICS_IN_LIST;

        | TopicTy                   | kind      | handler                       |
        | ----------                | ----      | -------                       |
    };
    topics_out: {
        list: TOPICS_OUT_LIST;
    };
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut peripheral_config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        peripheral_config.rcc.hsi = Some(HSIPrescaler::DIV1);
        peripheral_config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV16,
            mul: PllMul::MUL200,
            divp: Some(PllDiv::DIV2), 
            divq: Some(PllDiv::DIV2),
            divr: Some(PllDiv::DIV2),
        });
        peripheral_config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true }); 
        peripheral_config.rcc.sys = Sysclk::PLL1_P;
        peripheral_config.rcc.ahb_pre = AHBPrescaler::DIV2;
        peripheral_config.rcc.apb1_pre = APBPrescaler::DIV2;
        peripheral_config.rcc.apb2_pre = APBPrescaler::DIV2;
        peripheral_config.rcc.apb3_pre = APBPrescaler::DIV2;
        peripheral_config.rcc.apb4_pre = APBPrescaler::DIV2;

        peripheral_config.rcc.mux.usbsel = mux::Usbsel::HSI48; 
    }

    let mut p = embassy_stm32::init(peripheral_config);

    const USB_BUF_LEN: usize = 256;
    static USB_BUFFER: StaticCell<[u8; USB_BUF_LEN]> = StaticCell::new();
    let usb_buffer = USB_BUFFER.init([0u8; USB_BUF_LEN]);

    use static_cell::StaticCell;
    let driver = usb::Driver::new_hs(p.USB_OTG_HS, Irqs,p.PA12,p.PA11,usb_buffer,Default::default());
    let pbufs = PBUFS.take();
    let config = usb_config();

    let context = Context;
    let (device, tx_impl, rx_impl) = STORAGE.init(driver, config, pbufs.tx_buf.as_mut_slice());
    let dispatcher = MyApp::new(context, spawner.into());
    let vkk = dispatcher.min_key_len();
    let mut server: AppServer = Server::new(
        tx_impl,
        rx_impl,
        pbufs.rx_buf.as_mut_slice(),
        dispatcher,
        vkk,
    );
    spawner.must_spawn(usb_task(device));

    loop {
        // If the host disconnects, we'll return an error here.
        // If this happens, just wait until the host reconnects
        let _ = server.run().await;
        Timer::after_millis(10).await;
    }
}


#[embassy_executor::task]
pub async fn usb_task(mut usb: UsbDevice<'static, AppDriver>) {

    usb.run().await;
}

fn ping_handler(_context: &mut Context, _header: VarHeader, rqst: u32) -> u32 {
    info!("ping");
    rqst
}
