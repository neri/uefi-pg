// A Computer System

use super::super::scheduler::*;
use super::cpu::*;
use crate::*;
use alloc::boxed::Box;
use alloc::vec::*;
use core::ptr::NonNull;

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ProcessorId(pub u8);

impl ProcessorId {
    pub const fn as_u32(&self) -> u32 {
        self.0 as u32
    }
}

impl From<u8> for ProcessorId {
    fn from(val: u8) -> Self {
        Self(val)
    }
}

impl From<u32> for ProcessorId {
    fn from(val: u32) -> Self {
        Self(val as u8)
    }
}

impl From<usize> for ProcessorId {
    fn from(val: usize) -> Self {
        Self(val as u8)
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ProcessorIndex(pub usize);

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct VirtualAddress(pub usize);

impl VirtualAddress {
    pub const NULL: VirtualAddress = VirtualAddress(0);

    pub fn into_nonnull<T>(&self) -> Option<NonNull<T>> {
        if *self != Self::NULL {
            NonNull::new(self.0 as *const T as *mut T)
        } else {
            None
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct LockIrqHandle(pub usize);

pub struct System {
    total_memory_size: u64,
    number_of_cpus: usize,
    cpus: Vec<Box<Cpu>>,
    acpi: Option<Box<acpi::Acpi>>,
}

static mut SYSTEM: System = System::new();

unsafe impl Sync for System {}

impl System {
    const fn new() -> Self {
        System {
            total_memory_size: 0,
            number_of_cpus: 0,
            cpus: Vec::new(),
            acpi: None,
        }
    }

    pub unsafe fn init<F>(rsdptr: usize, total_memory_size: u64, f: F) -> !
    where
        F: FnOnce(&System) -> (),
    {
        let mut my_handler = MyAcpiHandler::new();

        SYSTEM.total_memory_size = total_memory_size;
        SYSTEM.acpi = Some(Box::new(acpi::parse_rsdp(&mut my_handler, rsdptr).unwrap()));
        SYSTEM.number_of_cpus = SYSTEM.acpi().application_processors.len() + 1;

        SYSTEM.cpus.push(Cpu::new(ProcessorId::from(
            SYSTEM.acpi().boot_processor.unwrap().local_apic_id,
        )));
        Cpu::init();

        GlobalScheduler::start_threading(&SYSTEM);

        myos::bus::lpc::LowPinCount::init();

        f(Self::shared());

        loop {
            Cpu::halt();
        }
    }

    #[inline]
    pub fn shared() -> &'static System {
        unsafe { &SYSTEM }
    }

    #[inline]
    pub fn number_of_cpus(&self) -> usize {
        self.number_of_cpus
    }

    #[inline]
    pub fn number_of_active_cpus(&self) -> usize {
        self.cpus.len()
    }

    #[inline]
    pub fn cpu(&self, index: usize) -> &Box<Cpu> {
        &self.cpus[index]
    }

    #[inline]
    pub fn total_memory_size(&self) -> u64 {
        self.total_memory_size
    }

    #[inline]
    pub fn acpi(&self) -> &acpi::Acpi {
        self.acpi.as_ref().unwrap()
    }

    pub fn current_cpu_index(&self) -> Option<usize> {
        unsafe {
            let handle = Cpu::lock_irq();
            let id = Cpu::current_processor_id();
            for (index, cpu) in self.cpus.iter().enumerate() {
                if cpu.cpu_id == id {
                    Cpu::unlock_irq(handle);
                    return Some(index);
                }
            }
            Cpu::unlock_irq(handle);
            None
        }
    }

    #[inline]
    pub(crate) unsafe fn activate_cpu(&self, new_cpu: Box<Cpu>) {
        SYSTEM.cpus.push(new_cpu);
    }
}

struct MyAcpiHandler {}

impl MyAcpiHandler {
    fn new() -> Self {
        MyAcpiHandler {}
    }
}

use acpi::handler::PhysicalMapping;
impl acpi::handler::AcpiHandler for MyAcpiHandler {
    unsafe fn map_physical_region<T>(
        &mut self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<T> {
        PhysicalMapping::<T> {
            physical_start: physical_address,
            virtual_start: NonNull::new(physical_address as *mut T).unwrap(),
            region_length: size,
            mapped_length: size,
        }
    }
    fn unmap_physical_region<T>(&mut self, _region: PhysicalMapping<T>) {}
}
