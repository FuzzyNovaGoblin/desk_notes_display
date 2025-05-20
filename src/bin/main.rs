// #![no_std]
// #![no_main]

// use core::net::Ipv4Addr;
// use core::str::from_utf8;

// use alloc::borrow::ToOwned;
// use embassy_futures::join::join;

// use embassy_executor::Spawner;
// use embassy_net::tcp::TcpSocket;
// use embassy_net::{Runner, StackResources};
// use embassy_time::{with_timeout, Duration, Timer};
// use esp_hal::clock::CpuClock;
// use esp_hal::rng::Rng;
// use esp_hal::time;
// use esp_hal::timer::systimer::SystemTimer;
// use esp_hal::timer::timg::TimerGroup;
// use esp_println::println;
// use esp_wifi::wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState};
// use esp_wifi::{init, EspWifiController};
// use smoltcp::iface::{SocketSet, SocketStorage};
// use smoltcp::wire::DhcpOption;
// use static_cell::make_static;

// const TEST_DURATION: usize = 15;
// const RX_BUFFER_SIZE: usize = 16384;
// const TX_BUFFER_SIZE: usize = 16384;
// const IO_BUFFER_SIZE: usize = 1024;
// const DOWNLOAD_PORT: u16 = 7272;
// const UPLOAD_PORT: u16 = 7272;
// const UPLOAD_DOWNLOAD_PORT: u16 = 4323;
// static mut RX_BUFFER: [u8; RX_BUFFER_SIZE] = [0; RX_BUFFER_SIZE];
// static mut TX_BUFFER: [u8; TX_BUFFER_SIZE] = [0; TX_BUFFER_SIZE];
// const HOST_IP: &str = "IP ADDRESS";

// macro_rules! mk_static {
//     ($t:ty,$val:expr) => {{
//         static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
//         #[deny(unused_attributes)]
//         let x = STATIC_CELL.uninit().write(($val));
//         x
//     }};
// }

// #[panic_handler]
// fn panic(_: &core::panic::PanicInfo) -> ! {
//     loop {}
// }

// extern crate alloc;

// const SSID: &str = env!("SSID");
// const PASSWORD: &str = env!("PASSWORD");

// #[esp_hal_embassy::main]
// async fn main(spawner: Spawner) {
//     // generator version: 0.3.1
//     esp_println::logger::init_logger_from_env();


//     let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
//     let peripherals = esp_hal::init(config);

//     esp_alloc::heap_allocator!(size: 72 * 1024);

//     let timer0 = SystemTimer::new(peripherals.SYSTIMER);
//     esp_hal_embassy::init(timer0.alarm0);

//     let timer1 = TimerGroup::new(peripherals.TIMG0);

//     let mut rng = Rng::new(peripherals.RNG);

//     let esp_wifi_ctrl = &*mk_static!(
//         EspWifiController<'static>,
//         init(timer1.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
//     );

//     // TODO: Spawn some tasks
//     let _ = spawner;

//     //start


//     // let esp_wifi_ctrl = &*mk_static!(
//     //     EspWifiController<'static>,
//     //     init(timg0.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
//     // );
//     let (mut controller, interfaces) =
//         esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI).unwrap();

//     let wifi_interface = interfaces.sta;

//     controller
//         .set_power_saving(esp_wifi::config::PowerSaveMode::None)
//         .unwrap();

//     // cfg_if::cfg_if! {
//     //     if #[cfg(feature = "esp32")] {
//     let timg1 = TimerGroup::new(peripherals.TIMG1);
//     esp_hal_embassy::init(timg1.timer0);
//     //     } else {
//     //         use esp_hal::timer::systimer::SystemTimer;
//     //         let systimer = SystemTimer::new(peripherals.SYSTIMER);
//     //         esp_hal_embassy::init(systimer.alarm0);
//     //     }
//     // }

//     let config = embassy_net::Config::dhcpv4(Default::default());

//     let seed = (rng.random() as u64) << 32 | rng.random() as u64;

//     // Init network stack
//     let (stack, runner) = embassy_net::new(
//         wifi_interface,
//         config,
//         mk_static!(StackResources<3>, StackResources::<3>::new()),
//         seed,
//     );

//     spawner.spawn(connection(controller)).ok();
//     spawner.spawn(net_task(runner)).ok();

//     println!("doing link up");
//     loop {
//         if stack.is_link_up() {
//             break;
//         }
//         Timer::after(Duration::from_millis(500)).await;
//     }
//     println!("link is up" );

//     println!("Waiting to get IP address...");
//     loop {
//         if let Some(config) = stack.config_v4() {
//             println!("Got IP: {}", config.address);
//             break;
//         }
//         Timer::after(Duration::from_millis(500)).await;
//     }

//     let mut socket = TcpSocket::new(
//         stack,
//         unsafe { &mut *core::ptr::addr_of_mut!(RX_BUFFER) },
//         unsafe { &mut *core::ptr::addr_of_mut!(TX_BUFFER) },
//     );

//     let server_address: Ipv4Addr = HOST_IP.parse().expect("Invalid HOST_IP address");

//     loop {
//         let _down = test_download(server_address, &mut socket).await;
//         let _up = test_upload(server_address, &mut socket).await;
//         let _updown = test_upload_download(server_address, &mut socket).await;

//         Timer::after(Duration::from_millis(10000)).await;
//     }

//     // let client_config = Configuration::Client(ClientConfiguration {
//     //     ssid,
//     //     password,
//     //     ..Default::default()
//     // });

//     // controller.set_configuration(&client_config).unwrap();

//     // loop {
//     //     println!("Hello, World!");
//     //     Timer::after(Duration::from_secs(1)).await;
//     // }

//     // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
// }

// #[embassy_executor::task]
// async fn connection(mut controller: WifiController<'static>) {
//     println!("start connection task");
//     println!("Device capabilities: {:?}", controller.capabilities());
//     loop {
//         match esp_wifi::wifi::wifi_state() {
//             WifiState::StaConnected => {
//                 // wait until we're no longer connected
//                 controller.wait_for_event(WifiEvent::StaDisconnected).await;
//                 Timer::after(Duration::from_millis(5000)).await
//             }
//             _ => {}
//         }
//         let ssid =
//             heapless::String::from_utf8(heapless::Vec::from_slice(&SSID.as_bytes()).unwrap())
//                 .unwrap();
//         let password =
//             heapless::String::from_utf8(heapless::Vec::from_slice(&PASSWORD.as_bytes()).unwrap())
//                 .unwrap();
//         if !matches!(controller.is_started(), Ok(true)) {
//             let client_config = Configuration::Client(ClientConfiguration {
//                 ssid: ssid.into(),
//                 password: password.into(),
//                 ..Default::default()
//             });
//             controller.set_configuration(&client_config).unwrap();
//             println!("Starting wifi");
//             controller.start_async().await.unwrap();
//             println!("Wifi started!");
//         }
//         println!("About to connect...");

//         match controller.connect_async().await {
//             Ok(_) => println!("Wifi connected!"),
//             Err(e) => {
//                 println!("Failed to connect to wifi: {e:?}");
//                 Timer::after(Duration::from_millis(5000)).await
//             }
//         }
//     }
// }

// #[embassy_executor::task]
// async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
//     runner.run().await
// }

// async fn test_download(server_address: Ipv4Addr, socket: &mut TcpSocket<'_>) -> usize {
//     println!("Testing download...");

//     socket.abort();
//     socket.set_timeout(Some(Duration::from_secs(10)));

//     println!("connecting to {:?}:{}...", server_address, DOWNLOAD_PORT);
//     if let Err(e) = socket.connect((server_address, DOWNLOAD_PORT)).await {
//         println!("connect error: {:?}", e);
//         return 0;
//     }
//     println!("connected, testing...");

//     let mut buf = [0; IO_BUFFER_SIZE];
//     let mut total: usize = 0;
//     with_timeout(Duration::from_secs(TEST_DURATION as _), async {
//         loop {
//             match socket.read(&mut buf).await {
//                 Ok(0) => {
//                     println!("read EOF");
//                     return 0;
//                 }
//                 Ok(n) => total += n,
//                 Err(e) => {
//                     println!("read error: {:?}", e);
//                     return 0;
//                 }
//             }
//         }
//     })
//     .await
//     .ok();

//     let kbps = (total + 512) / 1024 / TEST_DURATION;
//     println!("download: {} kB/s", kbps);
//     kbps
// }

// async fn test_upload(server_address: Ipv4Addr, socket: &mut TcpSocket<'_>) -> usize {
//     println!("Testing upload...");
//     socket.abort();
//     socket.set_timeout(Some(Duration::from_secs(10)));

//     println!("connecting to {:?}:{}...", server_address, UPLOAD_PORT);
//     if let Err(e) = socket.connect((server_address, UPLOAD_PORT)).await {
//         println!("connect error: {:?}", e);
//         return 0;
//     }
//     println!("connected, testing...");

//     let buf = [0; IO_BUFFER_SIZE];
//     let mut total: usize = 0;
//     with_timeout(Duration::from_secs(TEST_DURATION as _), async {
//         loop {
//             match socket.write(&buf).await {
//                 Ok(0) => {
//                     println!("write zero?!??!?!");
//                     return 0;
//                 }
//                 Ok(n) => total += n,
//                 Err(e) => {
//                     println!("write error: {:?}", e);
//                     return 0;
//                 }
//             }
//         }
//     })
//     .await
//     .ok();

//     let kbps = (total + 512) / 1024 / TEST_DURATION;
//     println!("upload: {} kB/s", kbps);
//     kbps
// }

// async fn test_upload_download(server_address: Ipv4Addr, socket: &mut TcpSocket<'_>) -> usize {
//     println!("Testing upload+download...");

//     socket.abort();
//     socket.set_timeout(Some(Duration::from_secs(10)));

//     println!(
//         "connecting to {:?}:{}...",
//         server_address, UPLOAD_DOWNLOAD_PORT
//     );
//     if let Err(e) = socket.connect((server_address, UPLOAD_DOWNLOAD_PORT)).await {
//         println!("connect error: {:?}", e);
//         return 0;
//     }
//     println!("connected, testing...");

//     let (mut reader, mut writer) = socket.split();

//     let tx_buf = [0; IO_BUFFER_SIZE];
//     let mut rx_buf = [0; IO_BUFFER_SIZE];
//     let mut total: usize = 0;
//     let tx_fut = async {
//         loop {
//             match writer.write(&tx_buf).await {
//                 Ok(0) => {
//                     println!("write zero?!??!?!");
//                     return 0;
//                 }
//                 Ok(_) => {}
//                 Err(e) => {
//                     println!("write error: {:?}", e);
//                     return 0;
//                 }
//             }
//         }
//     };

//     let rx_fut = async {
//         loop {
//             match reader.read(&mut rx_buf).await {
//                 Ok(0) => {
//                     println!("read EOF");
//                     return 0;
//                 }
//                 Ok(n) => total += n,
//                 Err(e) => {
//                     println!("read error: {:?}", e);
//                     return 0;
//                 }
//             }
//         }
//     };

//     with_timeout(
//         Duration::from_secs(TEST_DURATION as _),
//         join(tx_fut, rx_fut),
//     )
//     .await
//     .ok();

//     let kbps = (total + 512) / 1024 / TEST_DURATION;
//     println!("upload+download: {} kB/s", kbps);
//     kbps
// }

//! Run a test of download, upload and download+upload in async fashion.
//!
//! A prerequisite to running the benchmark examples is to run the benchmark server on your local machine. Simply run the following commands to do so.
//! ```
//! cd extras/bench-server
//! cargo run --release
//! ```
//! Ensure you have set the IP of your local machine in the `HOST_IP` env variable. E.g `HOST_IP="192.168.0.24"` and also set SSID and PASSWORD env variable before running this example.
//!
//! Because of the huge task-arena size configured this won't work on ESP32-S2 and ESP32-C2
//!

//% FEATURES: embassy esp-wifi esp-wifi/wifi esp-hal/unstable
//% CHIPS: esp32 esp32s2 esp32s3 esp32c3 esp32c6

#![allow(static_mut_refs)]
#![no_std]
#![no_main]

use core::net::Ipv4Addr;

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_net::{Runner, StackResources, tcp::TcpSocket};
use embassy_time::{Duration, Timer, with_timeout};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_println::println;
use esp_wifi::{
    EspWifiController,
    init,
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
};

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
const HOST_IP: &str = "IP ADDRESS";

const TEST_DURATION: usize = 15;
const RX_BUFFER_SIZE: usize = 16384;
const TX_BUFFER_SIZE: usize = 16384;
const IO_BUFFER_SIZE: usize = 1024;
const DOWNLOAD_PORT: u16 = 7272;
const UPLOAD_PORT: u16 = 7272;
const UPLOAD_DOWNLOAD_PORT: u16 = 7272;

// static buffers to not need a huge task-arena
static mut RX_BUFFER: [u8; RX_BUFFER_SIZE] = [0; RX_BUFFER_SIZE];
static mut TX_BUFFER: [u8; TX_BUFFER_SIZE] = [0; TX_BUFFER_SIZE];

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

    loop {
        let _down = test_download(server_address, &mut socket).await;
        let _up = test_upload(server_address, &mut socket).await;
        let _updown = test_upload_download(server_address, &mut socket).await;

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

async fn test_download(server_address: Ipv4Addr, socket: &mut TcpSocket<'_>) -> usize {
    println!("Testing download...");

    socket.abort();
    socket.set_timeout(Some(Duration::from_secs(10)));

    println!("connecting to {:?}:{}...", server_address, DOWNLOAD_PORT);
    if let Err(e) = socket.connect((server_address, DOWNLOAD_PORT)).await {
        println!("connect error: {:?}", e);
        return 0;
    }
    println!("connected, testing...");

    let mut buf = [0; IO_BUFFER_SIZE];
    let mut total: usize = 0;
    with_timeout(Duration::from_secs(TEST_DURATION as _), async {
        loop {
            match socket.read(&mut buf).await {
                Ok(0) => {
                    println!("read EOF");
                    return 0;
                }
                Ok(n) => total += n,
                Err(e) => {
                    println!("read error: {:?}", e);
                    return 0;
                }
            }
        }
    })
    .await
    .ok();

    let kbps = (total + 512) / 1024 / TEST_DURATION;
    println!("download: {} kB/s", kbps);
    kbps
}

async fn test_upload(server_address: Ipv4Addr, socket: &mut TcpSocket<'_>) -> usize {
    println!("Testing upload...");
    socket.abort();
    socket.set_timeout(Some(Duration::from_secs(10)));

    println!("connecting to {:?}:{}...", server_address, UPLOAD_PORT);
    if let Err(e) = socket.connect((server_address, UPLOAD_PORT)).await {
        println!("connect error: {:?}", e);
        return 0;
    }
    println!("connected, testing...");

    let buf = [0; IO_BUFFER_SIZE];
    let mut total: usize = 0;
    with_timeout(Duration::from_secs(TEST_DURATION as _), async {
        loop {
            match socket.write(&buf).await {
                Ok(0) => {
                    println!("write zero?!??!?!");
                    return 0;
                }
                Ok(n) => total += n,
                Err(e) => {
                    println!("write error: {:?}", e);
                    return 0;
                }
            }
        }
    })
    .await
    .ok();

    let kbps = (total + 512) / 1024 / TEST_DURATION;
    println!("upload: {} kB/s", kbps);
    kbps
}

async fn test_upload_download(server_address: Ipv4Addr, socket: &mut TcpSocket<'_>) -> usize {
    println!("Testing upload+download...");

    socket.abort();
    socket.set_timeout(Some(Duration::from_secs(10)));

    println!(
        "connecting to {:?}:{}...",
        server_address, UPLOAD_DOWNLOAD_PORT
    );
    if let Err(e) = socket.connect((server_address, UPLOAD_DOWNLOAD_PORT)).await {
        println!("connect error: {:?}", e);
        return 0;
    }
    println!("connected, testing...");

    let (mut reader, mut writer) = socket.split();

    let tx_buf = [0; IO_BUFFER_SIZE];
    let mut rx_buf = [0; IO_BUFFER_SIZE];
    let mut total: usize = 0;
    let tx_fut = async {
        loop {
            match writer.write(&tx_buf).await {
                Ok(0) => {
                    println!("write zero?!??!?!");
                    return 0;
                }
                Ok(_) => {}
                Err(e) => {
                    println!("write error: {:?}", e);
                    return 0;
                }
            }
        }
    };

    let rx_fut = async {
        loop {
            match reader.read(&mut rx_buf).await {
                Ok(0) => {
                    println!("read EOF");
                    return 0;
                }
                Ok(n) => total += n,
                Err(e) => {
                    println!("read error: {:?}", e);
                    return 0;
                }
            }
        }
    };

    with_timeout(
        Duration::from_secs(TEST_DURATION as _),
        join(tx_fut, rx_fut),
    )
    .await
    .ok();

    let kbps = (total + 512) / 1024 / TEST_DURATION;
    println!("upload+download: {} kB/s", kbps);
    kbps
}
