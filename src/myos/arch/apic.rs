// Advanced Programmable Interrupt Controller

use super::cpu::*;
use super::msr::Msr;
use super::x86_64::*;

use crate::myos::io::console::*;
use crate::myos::io::graphics::*;
use crate::stdout;

const IRQ_BASE: InterruptVector = InterruptVector(0x40);
const IRQ_LAPIC_TIMER: InterruptVector = InterruptVector(IRQ_BASE.0);

pub struct Apic {}

impl Apic {
    pub unsafe fn new(acpi_apic: &acpi::interrupt::Apic) -> Self {
        if acpi_apic.also_has_legacy_pics {
            // disable legacy PICs
            Cpu::out8(0xA1, 0xFF);
            Cpu::out8(0x21, 0xFF);
        }

        // disable IRQ
        Cpu::disable();

        LocalApic::init(acpi_apic.local_apic_address as usize);

        // enable IRQ
        Cpu::enable();

        // panic!("APIC {:?}", acpi_apic);
        Apic {}
    }
}

static mut LOCAL_APIC: LocalApic = LocalApic {
    base: core::ptr::null_mut(),
};

pub struct LocalApic {
    base: *mut u8,
}

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum LocalApicRegister {
    Id = 0x20,
    Version = 0x30,
    TaskPriority = 0x80,
    Eoi = 0xB0,
    SpuriousInterrupt = 0xF0,
    InterruptCommand = 0x300,
    InterruptCommandHigh = 0x310,
    LvtTimer = 0x320,
    LvtLint0 = 0x350,
    LvtLint1 = 0x360,
    LveError = 0x370,
    TimerInitialCount = 0x380,
    TimerCurrentCount = 0x390,
    TimerDivideConfiguration = 0x3E0,
}

impl LocalApicRegister {
    pub unsafe fn read(&self) -> u32 {
        LOCAL_APIC.read(*self)
    }

    pub unsafe fn write(&self, data: u32) {
        LOCAL_APIC.write(*self, data);
    }
}

impl LocalApic {
    const IA32_APIC_BASE_MSR_BSP: u64 = 0x00000100;
    const IA32_APIC_BASE_MSR_ENABLE: u64 = 0x00000800;

    unsafe fn init(base: usize) {
        let ptr = base as *const u8 as *mut u8;

        LOCAL_APIC.base = ptr;

        let msr = Msr::Ia32ApicBase;
        let val = msr.read();
        msr.write(
            (val & Self::IA32_APIC_BASE_MSR_BSP)
                | ((base as u64 & !0xFFF) | Self::IA32_APIC_BASE_MSR_ENABLE),
        );

        InterruptDescriptorTable::register(
            IRQ_LAPIC_TIMER,
            LinearAddress(timer_handler as usize as u64),
        );

        LocalApicRegister::TimerDivideConfiguration.write(0x0000000B);
        // LocalApicRegister::LvtTimer.write(0x00010020);
        LocalApicRegister::LvtTimer.write(0x00020000 | IRQ_LAPIC_TIMER.0 as u32);
        LocalApicRegister::TimerInitialCount.write(0x100000);
    }

    unsafe fn read(&self, index: LocalApicRegister) -> u32 {
        let ptr = self.base.add(index as usize) as *const u32;
        ptr.read_volatile()
    }

    unsafe fn write(&self, index: LocalApicRegister, value: u32) {
        let ptr = self.base.add(index as usize) as *const u32 as *mut u32;
        ptr.write_volatile(value);
    }

    unsafe fn eoi(&self) {
        self.write(LocalApicRegister::Eoi, 0);
    }

    unsafe fn read_id(&self) -> u32 {
        self.read(LocalApicRegister::Id) >> 24
    }
}

static mut TIMER_COUNTER: usize = 0;

extern "x86-interrupt" fn timer_handler(_stack_frame: &ExceptionStackFrame) {
    unsafe {
        TIMER_COUNTER += 0x123456;
        stdout()
            .fb()
            .fill_rect(Rect::new(30, 30, 20, 20), Color::from(TIMER_COUNTER as u32));
        LOCAL_APIC.eoi();
    }
}
