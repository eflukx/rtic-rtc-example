#![no_main]
#![no_std]

use rtic_nrf_rtc as _; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    rtic_nrf_rtc::exit()
}
