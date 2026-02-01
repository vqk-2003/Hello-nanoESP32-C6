#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Level;
use esp_hal::rmt::{Channel, PulseCode, Rmt, Tx, TxChannelConfig, TxChannelCreator};
use esp_hal::time::Rate;
use esp_hal::{Blocking, main};

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(5)).unwrap();
    let mut channel = rmt
        .channel0
        .configure_tx(
            peripherals.GPIO8,
            TxChannelConfig::default()
                .with_clk_divider(1)
                .with_idle_output(true),
        )
        .unwrap();

    let delay = Delay::new();
    let mut r = 10;
    let mut g = 10;
    let mut b = 10;
    let mut index = 0;
    let mut led = Ws2812b::init(r, g, b);

    loop {
        delay.delay_millis(500);
        match index {
            0 => r ^= 10,
            1 => g ^= 10,
            _ => b ^= 10,
        }
        led.change_led(r, g, b);
        channel = led.update(channel);
        index = if (index + 1) > 2 { 0 } else { index + 1 };
    }
}

struct Ws2812b {
    data: [PulseCode; 25],
}

const LOGIC_ONE: PulseCode = PulseCode::new(Level::High, 4, Level::Low, 2);
const LOGIC_ZERO: PulseCode = PulseCode::new(Level::High, 2, Level::Low, 4);

impl Ws2812b {
    fn init(r: u8, g: u8, b: u8) -> Self {
        let mut led = Self {
            data: [PulseCode::end_marker(); 25],
        };
        led.change_led(r, g, b);
        led
    }

    fn change_led(&mut self, mut r: u8, mut g: u8, mut b: u8) {
        for index in 0..24 {
            if index < 8 {
                self.data[index] = if (g & 0x80) != 0 {
                    LOGIC_ONE
                } else {
                    LOGIC_ZERO
                };
                g <<= 1;
            } else if index < 16 {
                self.data[index] = if (r & 0x80) != 0 {
                    LOGIC_ONE
                } else {
                    LOGIC_ZERO
                };
                r <<= 1;
            } else {
                self.data[index] = if (b & 0x80) != 0 {
                    LOGIC_ONE
                } else {
                    LOGIC_ZERO
                };
                b <<= 1;
            }
        }
    }

    fn update<'a>(&mut self, channel: Channel<'a, Blocking, Tx>) -> Channel<'a, Blocking, Tx> {
        channel.transmit(&self.data).unwrap().wait().unwrap()
    }
}
