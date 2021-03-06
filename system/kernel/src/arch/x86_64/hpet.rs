// High Precision Event Timer

use super::{apic::*, page::PhysicalAddress};
use crate::{mem::mmio::*, task::scheduler::*, *};
use alloc::boxed::Box;
use core::time::Duration;

pub(super) struct Hpet {
    mmio: Mmio,
    main_cnt_period: u64,
    measure_div: u64,
}

impl Hpet {
    pub unsafe fn new(info: &acpi::HpetInfo) -> Box<Self> {
        let mut hpet = Hpet {
            mmio: Mmio::from_phys(info.base_address as PhysicalAddress, 0x1000).unwrap(),
            main_cnt_period: 0,
            measure_div: 0,
        };

        Irq::LPC_TIMER.register(Self::irq_handler).unwrap();

        hpet.main_cnt_period = hpet.read(0) >> 32;
        hpet.write(0x10, 0);
        hpet.write(0x20, 0); // Clear all interrupts
        hpet.write(0xF0, 0); // Reset MAIN_COUNTER_VALUE
        hpet.write(0x10, 0x03); // LEG_RT_CNF | ENABLE_CNF

        hpet.measure_div = 1000_000_000 / hpet.main_cnt_period;
        hpet.write(0x100, 0x0000_004C); // Tn_INT_ENB_CNF | Tn_TYPE_CNF | Tn_VAL_SET_CNF
        hpet.write(0x108, 1000_000_000_000 / hpet.main_cnt_period);

        Box::new(hpet)
    }

    unsafe fn read(&self, index: usize) -> u64 {
        self.mmio.read_u64(index)
    }

    unsafe fn write(&self, index: usize, value: u64) {
        self.mmio.write_u64(index, value);
    }

    /// IRQ of HPET
    /// Currently, this system does not require an IRQ for HPET, but it is receiving an interrupt just in case.
    fn irq_handler(_irq: Irq) {
        // TODO:
    }
}

impl TimerSource for Hpet {
    fn measure(&self) -> TimeSpec {
        TimeSpec((unsafe { self.read(0xF0) } / self.measure_div) as usize)
    }

    fn from_duration(&self, val: Duration) -> TimeSpec {
        TimeSpec(val.as_micros() as usize)
    }

    fn to_duration(&self, val: TimeSpec) -> Duration {
        Duration::from_micros(val.0 as u64)
    }
}
