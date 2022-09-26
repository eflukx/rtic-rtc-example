// RTIC Monotonic impl for the RTCs
// for reference, see: https://gist.github.com/korken89/fe94a475726414dd1bce031c76adc3dd

use crate::hal::pac::{rtc0, RTC0, RTC1, RTC2};
pub use fugit::{self, ExtU64};
use rtic_monotonic::Monotonic;

pub struct MonoRtc<T: InstanceRtc> {
    overflow: u64,
    rtc: T,
}

impl<T: InstanceRtc> MonoRtc<T> {
    pub fn new(rtc: T) -> Self {
        unsafe { rtc.prescaler.write(|w| w.bits(0)) };
        MonoRtc { overflow: 0, rtc }
    }

    pub fn is_overflow(&self) -> bool {
        self.rtc.events_ovrflw.read().bits() == 1
    }
}

impl<T: InstanceRtc> Monotonic for MonoRtc<T> {
    type Instant = fugit::TimerInstantU64<32_768>;
    type Duration = fugit::TimerDurationU64<32_768>;

    const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = false;

    unsafe fn reset(&mut self) {
        self.rtc.intenset.write(|w| w.compare0().set().ovrflw().set());
        self.rtc.evtenset.write(|w| w.compare0().set().ovrflw().set());

        self.rtc.tasks_clear.write(|w| w.bits(1));
        self.rtc.tasks_start.write(|w| w.bits(1));
    }

    #[inline(always)]
    fn now(&mut self) -> Self::Instant {
        let cnt = self.rtc.counter.read().bits();
        let ovf = if self.is_overflow() { self.overflow.wrapping_add(1) } else { self.overflow };

        Self::Instant::from_ticks((ovf << 24) | cnt as u64)
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        let now = self.now();

        // A minimum amount of ticks left is needed (empirically found this to be 3)
        // if ticks left is lower than this bound, the interrupt may not fire (and hang the app).
        const MIN_TICKS_FOR_COMPARE: u64 = 3;

        // Since the timer may or may not overflow based on the requested compare val, we check
        // how many ticks are left.
        let val = match instant.checked_duration_since(now) {
            Some(x) if (x.ticks() <= 0x00ff_ffff && x.ticks() > MIN_TICKS_FOR_COMPARE) => {
                instant.duration_since_epoch().ticks() & 0x00ff_ffff
            }

            Some(x) => (instant.duration_since_epoch().ticks() + (MIN_TICKS_FOR_COMPARE - x.ticks())) & 0x00ff_ffff,

            _ => 0, // Will overflow or in the past, set the same value as after overflow to not get extra interrupts
        } as u32;

        unsafe { self.rtc.cc[0].write(|w| w.bits(val)) };
    }

    fn clear_compare_flag(&mut self) {
        unsafe { self.rtc.events_compare[0].write(|w| w.bits(0)) };
    }

    #[inline(always)]
    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }

    fn on_interrupt(&mut self) {
        if self.is_overflow() {
            self.overflow = self.overflow.wrapping_add(1);
            self.rtc.events_ovrflw.write(|w| unsafe { w.bits(0) });
        }
    }
}

pub trait InstanceRtc: core::ops::Deref<Target = rtc0::RegisterBlock> {}
impl InstanceRtc for RTC0 {}
impl InstanceRtc for RTC1 {}
impl InstanceRtc for RTC2 {}
