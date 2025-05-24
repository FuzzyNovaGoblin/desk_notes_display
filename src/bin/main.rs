#![allow(static_mut_refs)]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

use core::{fmt::Display, net::Ipv4Addr};

use desktop_notes_display::example_display::ExampleDisplay;
use embassy_executor::Spawner;
use embassy_net::{tcp::TcpSocket, Runner, StackResources};
use embassy_time::{with_timeout, Duration, Timer};
use embedded_graphics::{
    mock_display::MockDisplay,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::{BinaryColor, Gray8},
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers,
    rng::Rng,
    spi::{
        self,
        master::{Config, Spi},
    },
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::{dbg, print, println};
use esp_wifi::{
    init,
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
    EspWifiController,
};
use heapless::{String, Vec};

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");
const HOST_IP: &str = env!("IPADDR");

const X_BUFFER_SIZE: usize = 16384;
const IO_BUFFER_SIZE: usize = 1024;
const DOWNLOAD_PORT: u16 = 7272;

// static buffers to not need a huge task-arena
static mut RX_BUFFER: [u8; X_BUFFER_SIZE] = [0; X_BUFFER_SIZE];
static mut TX_BUFFER: [u8; X_BUFFER_SIZE] = [0; X_BUFFER_SIZE];

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 32 * 1024);
    // add some more RAM
    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 64 * 1024);

    let server_address: Ipv4Addr = HOST_IP.parse().expect("Invalid HOST_IP address");

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let mut rng = Rng::new(peripherals.RNG);

    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(timg0.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
    );

    let (mut controller, interfaces) =
        esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI).unwrap();

    let wifi_interface = interfaces.sta;

    controller
        .set_power_saving(esp_wifi::config::PowerSaveMode::None)
        .unwrap();

    use esp_hal::timer::systimer::SystemTimer;
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);
    // cfg_if::cfg_if! {
    //     if #[cfg(feature = "esp32")] {
    //         let timg1 = TimerGroup::new(peripherals.TIMG1);
    //         esp_hal_embassy::init(timg1.timer0);
    //     } else {
    //         use esp_hal::timer::systimer::SystemTimer;
    //         let systimer = SystemTimer::new(peripherals.SYSTIMER);
    //         esp_hal_embassy::init(systimer.alarm0);
    //     }
    // }

    let config = embassy_net::Config::dhcpv4(Default::default());

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    // setup display

    /*
    RST    D0
    CS	   D1
    DC	   D3
    BUSY   D5
    SCK	   D8
    MOSI   D10
     */
    let rst = peripherals.GPIO2;
    let cs = peripherals.GPIO3;
    let dc = peripherals.GPIO5;
    let busy = peripherals.GPIO7;
    let sclk = peripherals.GPIO8;
    let mosi = peripherals.GPIO10;

    // let miso = peripherals.GPIO;
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let mut spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_khz(100))
            .with_mode(spi::Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi)
    // .with_miso(miso)
    .with_cs(cs)
    .with_dma(peripherals.DMA_CH0)
    .with_buffers(dma_rx_buf, dma_tx_buf)
    .into_async();

    // embeded graphics

    let mut display = ExampleDisplay::new([0; 64*63], spi);
    dbg!();

    // Draw a circle with top-left at `(22, 22)` with a diameter of `20` and a white stroke
    let circle = Circle::new(Point::new(22, 22), 20)
        .into_styled(PrimitiveStyle::with_stroke(Gray8::WHITE, 1));
    dbg!();

    circle.draw(&mut display).unwrap();
    dbg!();

    // Update the display
    display.flush().unwrap();

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    let mut socket = TcpSocket::new(
        stack,
        unsafe { &mut *core::ptr::addr_of_mut!(RX_BUFFER) },
        unsafe { &mut *core::ptr::addr_of_mut!(TX_BUFFER) },
    );

    // let mut display = MockDisplay::new();

    // // Create styles used by the drawing operations.
    // let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    // let thick_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 3);
    // let border_stroke = PrimitiveStyleBuilder::new()
    //     .stroke_color(BinaryColor::On)
    //     .stroke_width(3)
    //     .stroke_alignment(StrokeAlignment::Inside)
    //     .build();
    // let fill = PrimitiveStyle::with_fill(BinaryColor::On);
    // let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // let yoffset = 10;

    // // Draw a 3px wide outline around the display.
    // display
    //     .bounding_box()
    //     .into_styled(border_stroke)
    //     .draw(&mut display).unwrap();

    // // Draw a triangle.
    // Triangle::new(
    //     Point::new(16, 16 + yoffset),
    //     Point::new(16 + 16, 16 + yoffset),
    //     Point::new(16 + 8, yoffset),
    // )
    // .into_styled(thin_stroke)
    // .draw(&mut display).unwrap();

    // // Draw a filled square
    // Rectangle::new(Point::new(52, yoffset), Size::new(16, 16))
    //     .into_styled(fill)
    //     .draw(&mut display).unwrap();

    // // Draw a circle with a 3px wide stroke.
    // Circle::new(Point::new(88, yoffset), 17)
    //     .into_styled(thick_stroke)
    //     .draw(&mut display).unwrap();

    // // Draw centered text.
    // let text = "embedded-graphics";
    // Text::with_alignment(
    //     text,
    //     display.bounding_box().center() + Point::new(0, 15),
    //     character_style,
    //     Alignment::Center,
    // )
    // .draw(&mut display).unwrap();

    loop {
        let _down = download_data(server_address, &mut socket).await;

        Timer::after(Duration::from_millis(10000)).await;
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: heapless::String::try_from(SSID).unwrap(),
                password: heapless::String::try_from(PASSWORD).unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start_async().await.unwrap();
            println!("Wifi started!");
        }
        println!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

async fn download_data(server_address: Ipv4Addr, socket: &mut TcpSocket<'_>) {
    println!("Testing download...");

    socket.abort();
    socket.set_timeout(Some(Duration::from_secs(10)));

    println!("connecting to {:?}:{}...", server_address, DOWNLOAD_PORT);
    if let Err(e) = socket.connect((server_address, DOWNLOAD_PORT)).await {
        println!("connect error: {:?}", e);
        return;
    }
    println!("connected, testing...");

    socket
        .write(b"GET / HTTP/1.0\r\nHost: `IPADDR`:7272\r\n\r\n")
        .await
        .unwrap();

    let mut buf: [u8; IO_BUFFER_SIZE] = [0; IO_BUFFER_SIZE];
    loop {
        let read_amt = socket.read(&mut buf).await.unwrap();
        print!(
            "{}",
            String::from_utf8(Vec::<u8, IO_BUFFER_SIZE>::from_slice(&buf).unwrap()).unwrap()
        );
        if read_amt < IO_BUFFER_SIZE {
            break;
        }
    }
    println!();
}
