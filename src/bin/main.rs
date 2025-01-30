#![no_std]
#![no_main]

use alloc::format;
use embedded_graphics::prelude::*;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::Point,
    text::{Baseline, Text},
};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::Config;
use esp_hal::{main, time};
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use esp_wifi::init;
use esp_wifi::wifi::utils::create_network_interface;
use esp_wifi::wifi::{ClientConfiguration, Configuration, WifiStaDevice};
use fugit::RateExtU32;
use log::info;
use smoltcp::{
    iface::{SocketSet, SocketStorage},
    wire::{DhcpOption, IpAddress},
};
use ssd1306::mode::DisplayConfig;
use ssd1306::{prelude::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface, Ssd1306};

extern crate alloc;

#[main]
fn main() -> ! {
    // generator version: 0.2.2

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let init = esp_wifi::init(
        timg0.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    /* START - setup wifi */
    let mut wifi = peripherals.WIFI;
    let (iface, device, mut controller) =
        create_network_interface(&init, &mut wifi, WifiStaDevice).unwrap();

    /* idk start */

    // let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    // let mut socket_set = SocketSet::new(&mut socket_set_entries[..]);
    // let mut dhcp_socket = smoltcp::socket::dhcpv4::Socket::new();
    // // we can set a hostname here (or add other DHCP options)
    // dhcp_socket.set_outgoing_options(&[DhcpOption {
    //     kind: 12,
    //     data: b"esp-wifi",
    // }]);
    // socket_set.add(dhcp_socket);

    // let now = || time::now().duration_since_epoch().to_millis();
    // let stack = Stack::new(iface, device, socket_set, now, rng.random());

    /* idk end */

    let client_config = Configuration::Client(ClientConfiguration {
        ssid: include_str!("wifi_ssid.in").try_into().unwrap(),
        password: include_str!("wifi_pass.in").try_into().unwrap(),
        ..Default::default()
    });

    let res = controller.set_configuration(&client_config);
    println!("wifi_set_configuration returned {:?}", res);

        controller.start().unwrap();
    println!("is wifi started: {:?}", controller.is_started());



    /* END - setup wifi */

    /* START - setup display */
    let i2c = esp_hal::i2c::master::I2c::new(
        peripherals.I2C0,
        // Config::default(),
        Config::default().with_frequency(400000_u32.Hz()),
    )
    .unwrap()
    .with_sda(peripherals.GPIO5)
    .with_scl(peripherals.GPIO6);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    /* END - setup display */

    // test start
    // let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
    // let res = client.get(URL)?.submit()?;
    // println!("{}", res);

    // test end

    let delay = Delay::new();
    let mut count = 0;
    loop {
        display.clear(BinaryColor::Off).unwrap();

        Text::with_baseline(
            format!("toot {}", count).as_str(),
            Point::zero(),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();
        display.flush().unwrap();
        // info!("Hello world!");

        info!("hello world");
        delay.delay_millis(10000);
        count += 1;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
