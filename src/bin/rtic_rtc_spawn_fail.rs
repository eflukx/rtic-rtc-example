#![no_main]
#![no_std]
#![feature(core_intrinsics)]

use hal::{clocks, Clocks};
use rtic_nrf_rtc::hal;

use fugit::ExtU64;
use rtic::app;

#[app(device = hal::pac, peripherals = true, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    use rtic_nrf_rtc::monotonic_nrf52_rtc::MonoRtc;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        hi_cnt: u32,
        first_fast: bool,
    }

    #[monotonic(binds = RTC1, default = true)]
    type Tonic = MonoRtc<hal::pac::RTC1>;
    type TonicDuration = <Tonic as rtic::Monotonic>::Duration;

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("RTIC RTC demo...");
        cx.core.SCB.set_sleepdeep();

        // Configure clocks as appropriate for your board.
        let _clocks = Clocks::new(cx.device.CLOCK)
            .set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass)
            .start_lfclk();
        defmt::debug!("Clocks configured");

        let mono = Tonic::new(cx.device.RTC1);

        if let Err(e) = hi_there::spawn() {
            defmt::error!("Eror spawning `hi_there` task! {:?}", e);
        }

        if let Err(e) = fast_task::spawn_after(3.secs()) {
            defmt::error!("Eror spawning `fast_task`! {:?}", e);
        }

        (
            Shared {},
            Local {
                first_fast: true,
                hi_cnt: 0,
            },
            init::Monotonics(mono),
        )
    }

    #[task(local = [first_fast])]
    fn fast_task(ctx: fast_task::Context) {
        if *ctx.local.first_fast {
            defmt::info!("fast_task first spawned!");
            *ctx.local.first_fast = false;
        }

        defmt::trace!("fast_task");

        fast_task::spawn_after(100.micros()).ok();                  // Fails quickly
        // fast_task::spawn_after(TonicDuration::from_ticks(1)).ok();  // Fails quickly
        // fast_task::spawn_after(1.secs()).ok();                      // Does not fail
        // fast_task::spawn_after(1.micros()).ok();                    // Does not fail (equates to 0 ticks, so is equal to a direct spawn I guess)
    }

    #[task(shared = [], local = [hi_cnt])]
    fn hi_there(ctx: hi_there::Context) {
        *ctx.local.hi_cnt += 1;
        let uptime = monotonics::now().duration_since_epoch().to_secs();

        defmt::info!(
            "Hi there! {} times, we're up since {} seconds",
            ctx.local.hi_cnt,
            uptime
        );

        hi_there::spawn_after(1.secs()).ok();
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            defmt::trace!("idling . . .");
            rtic::export::wfi();
        }
    }
}
