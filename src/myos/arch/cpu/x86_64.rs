// Central Processing Unit

use crate::myos::arch::apic::Apic;
use crate::myos::arch::system::*;
use alloc::boxed::Box;
use bitflags::*;

// #[derive(Debug)]
pub struct Cpu {
    pub cpu_id: ProcessorId,
    pub gdt: Box<GlobalDescriptorTable>,
    pub tss: Box<TaskStateSegment>,
}

//unsafe impl Sync for Cpu {}

impl Cpu {
    pub(crate) unsafe fn new(cpuid: ProcessorId) -> Box<Self> {
        let tss = TaskStateSegment::new();
        let gdt = GlobalDescriptorTable::new(&tss);
        let cpu = Box::new(Cpu {
            cpu_id: cpuid,
            gdt: gdt,
            tss: tss,
        });
        cpu
    }

    pub(crate) unsafe fn init() {
        InterruptDescriptorTable::init();

        if let acpi::InterruptModel::Apic(apic) =
            System::shared().acpi().interrupt_model.as_ref().unwrap()
        {
            Apic::init(apic);
        } else {
            panic!("NO APIC");
        }
    }

    pub fn current_processor_id() -> ProcessorId {
        Apic::current_processor_id()
    }

    pub fn relax() {
        unsafe {
            llvm_asm!("pause");
        }
    }

    pub unsafe fn halt() {
        llvm_asm!("hlt");
    }

    pub unsafe fn reset() -> ! {
        // io_out8(0x0CF9, 0x06);
        // moe_usleep(10000);
        Cpu::out8(0x0092, 0x01);
        loop {
            Cpu::halt()
        }
    }

    pub unsafe fn out8(port: u16, value: u8) {
        llvm_asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(value));
    }

    #[must_use]
    pub unsafe fn lock_irq() -> LockIrqHandle {
        let mut rax: Eflags;
        llvm_asm!("
            pushfq
            cli
            pop $0
            "
            : "=r"(rax));
        LockIrqHandle((rax & Eflags::IF).bits as usize)
    }

    pub unsafe fn unlock_irq(handle: LockIrqHandle) {
        let eflags = Eflags::from_bits_unchecked(handle.0);
        if eflags.contains(Eflags::IF) {
            llvm_asm!("sti");
        }
    }
}

bitflags! {
    pub struct Eflags: usize {
        const CF = 0x00000001;
        const PF = 0x00000004;
        const AF = 0x00000010;
        const ZF = 0x00000040;
        const SF = 0x00000080;
        const TF = 0x00000100;
        const IF = 0x00000200;
        const DF = 0x00000400;
        const OF = 0x00000800;
        // const IOPLMASK = 0x00003000;
        // const IOPL3 = IOPLMASK;
        const NT = 0x00004000;
        const RF = 0x00010000;
        const VM = 0x00020000;
        const AC = 0x00040000;
        const VIF = 0x00080000;
        const VIP = 0x00100000;
        const ID = 0x00200000;
    }
}

const MAX_GDT: usize = 8;
const MAX_IDT: usize = 256;
const KERNEL_CODE: Selector = Selector::new(1, PrivilegeLevel::Kernel);
const KERNEL_DATA: Selector = Selector::new(2, PrivilegeLevel::Kernel);
const TSS: Selector = Selector::new(6, PrivilegeLevel::Kernel);

use core::fmt;
impl fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtualAddress({:#016x})", self.0)
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Limit(pub u16);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Selector(pub u16);

impl Selector {
    pub const NULL: Selector = Selector(0);

    pub const fn new(index: usize, rpl: PrivilegeLevel) -> Self {
        Selector((index << 3) as u16 | rpl as u16)
    }

    pub fn rpl(&self) -> PrivilegeLevel {
        PrivilegeLevel::from(self.0 as usize)
    }

    pub const fn index(&self) -> usize {
        (self.0 >> 3) as usize
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum PrivilegeLevel {
    Kernel = 0,
    System1,
    System2,
    User,
}

impl PrivilegeLevel {
    pub const fn as_descriptor_entry(&self) -> u64 {
        let dpl = *self as u64;
        dpl << 13
    }
}

impl From<usize> for PrivilegeLevel {
    fn from(value: usize) -> PrivilegeLevel {
        match value & 3 {
            0 => PrivilegeLevel::Kernel,
            1 => PrivilegeLevel::System1,
            2 => PrivilegeLevel::System2,
            3 => PrivilegeLevel::User,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DescriptorType {
    Null = 0,
    Tss = 9,
    TssBusy = 11,
    InterruptGate = 14,
    TrapGate = 15,
}

impl DescriptorType {
    pub const fn as_descriptor_entry(&self) -> u64 {
        let ty = *self as u64;
        ty << 40
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct InterruptVector(pub u8);

// impl core::ops::Add<u8> for InterruptVector {
//     type Output = Self;
//     fn add(self, rhs: u8) -> Self {
//         Self(self.0 + rhs)
//     }
// }

// impl core::ops::Sub<u8> for InterruptVector {
//     type Output = Self;
//     fn sub(self, rhs: u8) -> Self {
//         Self(self.0 - rhs)
//     }
// }

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Exception {
    DivideError = 0,
    Debug = 1,
    NonMaskable = 2,
    Breakpoint = 3,
    Overflow = 4,
    //Deprecated = 5,
    InvalidOpcode = 6,
    DeviceNotAvailable = 7,
    DoubleFault = 8,
    //Deprecated = 9,
    InvalidTss = 10,
    SegmentNotPresent = 11,
    StackException = 12,
    GeneralProtection = 13,
    PageFault = 14,
    //Unavailable = 15,
    FloatingPointException = 16,
    AlignmentCheck = 17,
    MachineCheck = 18,
    SimdException = 19,
}

impl Exception {
    pub const fn as_vec(&self) -> InterruptVector {
        InterruptVector(*self as u8)
    }
}

impl From<Exception> for InterruptVector {
    fn from(ex: Exception) -> Self {
        InterruptVector(ex as u8)
    }
}

#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved_1: u32,
    pub stack_pointer: [u64; 3],
    reserved_2: u32,
    pub ist: [u64; 7],
    reserved_3: u64,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub fn new() -> Box<Self> {
        Box::new(TaskStateSegment {
            stack_pointer: [0; 3],
            ist: [0; 7],
            iomap_base: 0,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
        })
    }

    pub fn limit(&self) -> Limit {
        Limit(0x67)
    }
}

#[repr(u64)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DefaultSize {
    Use16 = 0x0000_0000_0000_0000,
    Use32 = 0x00C0_0000_0000_0000,
    Use64 = 0x00A0_0000_0000_0000,
}

impl DefaultSize {
    pub const fn as_descriptor_entry(&self) -> u64 {
        *self as u64
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq)]
pub struct DescriptorEntry(pub u64);

impl DescriptorEntry {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn present() -> u64 {
        0x8000_0000_0000
    }

    pub const fn code_segment(dpl: PrivilegeLevel, size: DefaultSize) -> Self {
        let value = 0x000F9A000000FFFFu64 | dpl.as_descriptor_entry() | size.as_descriptor_entry();
        DescriptorEntry(value)
    }

    pub const fn data_segment(dpl: PrivilegeLevel) -> Self {
        let value = 0x008F92000000FFFFu64 | dpl.as_descriptor_entry();
        DescriptorEntry(value)
    }

    pub const fn tss_descriptor(offset: VirtualAddress, limit: Limit) -> DescriptorPair {
        let offset = offset.0 as u64;
        let low = DescriptorEntry(
            limit.0 as u64
                | Self::present()
                | DescriptorType::Tss.as_descriptor_entry()
                | (offset & 0x00FF_FFFF) << 16
                | (offset & 0xFF00_0000) << 32,
        );
        let high = DescriptorEntry(offset >> 32);
        DescriptorPair::new(low, high)
    }

    pub const fn gate_descriptor(
        offset: VirtualAddress,
        sel: Selector,
        dpl: PrivilegeLevel,
        ty: DescriptorType,
    ) -> DescriptorPair {
        let offset = offset.0 as u64;
        let low = DescriptorEntry(
            (offset & 0xFFFF)
                | (sel.0 as u64) << 16
                | Self::present()
                | dpl.as_descriptor_entry()
                | ty.as_descriptor_entry()
                | (offset & 0xFFFF_0000) << 32,
        );
        let high = DescriptorEntry(offset >> 32);

        DescriptorPair::new(low, high)
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub struct DescriptorPair {
    pub low: DescriptorEntry,
    pub high: DescriptorEntry,
}

impl DescriptorPair {
    pub const fn new(low: DescriptorEntry, high: DescriptorEntry) -> Self {
        DescriptorPair {
            low: low,
            high: high,
        }
    }
}

#[repr(C, align(16))]
pub struct GlobalDescriptorTable {
    table: [DescriptorEntry; MAX_GDT],
}

impl GlobalDescriptorTable {
    pub fn new(tss: &Box<TaskStateSegment>) -> Box<Self> {
        let tss_pair = DescriptorEntry::tss_descriptor(
            VirtualAddress(tss.as_ref() as *const _ as usize),
            tss.limit(),
        );
        let mut gdt = Box::new(GlobalDescriptorTable {
            table: [DescriptorEntry::null(); MAX_GDT],
        });
        gdt.table[KERNEL_CODE.index()] =
            DescriptorEntry::code_segment(PrivilegeLevel::Kernel, DefaultSize::Use64);
        gdt.table[KERNEL_DATA.index()] = DescriptorEntry::data_segment(PrivilegeLevel::Kernel);
        let tss_index = TSS.index();
        gdt.table[tss_index] = tss_pair.low;
        gdt.table[tss_index + 1] = tss_pair.high;

        unsafe {
            gdt.reload();
        }
        gdt
    }

    // Reload GDT and Segment Selectors
    pub unsafe fn reload(&self) {
        llvm_asm!("
            push $0
            push $1
            lgdt 6(%rsp)
            add $$0x10, %rsp
            "
            ::"r"(&self.table), "r"((self.table.len() * 8 - 1) << 48)
        );
        llvm_asm!("
            mov %rsp, %rax
            push %rdx
            push %rax
            pushfq
            push %rcx
            .byte 0xE8, 2, 0, 0, 0, 0xEB, 0x02, 0x48, 0xCF
            mov %edx, %ds
            mov %edx, %es
            mov %edx, %fs
            mov %edx, %gs
            "
            ::"{rcx}"(KERNEL_CODE), "{rdx}"(KERNEL_DATA)
            :"%rax"
        );
        llvm_asm!("ltr $0"::"r"(TSS));
    }
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

#[repr(C, align(16))]
pub struct InterruptDescriptorTable {
    raw: [DescriptorEntry; MAX_IDT * 2],
}

impl InterruptDescriptorTable {
    const fn new() -> Self {
        InterruptDescriptorTable {
            raw: [DescriptorEntry::null(); MAX_IDT * 2],
        }
    }

    pub fn init() {
        unsafe {
            Self::load();
            Self::register(
                Exception::InvalidOpcode.as_vec(),
                VirtualAddress(interrupt_ud_handler as usize),
            );
            Self::register(
                Exception::DoubleFault.as_vec(),
                VirtualAddress(interrupt_df_handler as usize),
            );
            Self::register(
                Exception::GeneralProtection.as_vec(),
                VirtualAddress(interrupt_gp_handler as usize),
            );
            Self::register(
                Exception::PageFault.as_vec(),
                VirtualAddress(interrupt_page_handler as usize),
            );
        }
    }

    pub unsafe fn load() {
        llvm_asm!("
            push $0
            push $1
            lidt 6(%rsp)
            add $$0x10, %rsp
            "
            :: "r"(&IDT.raw), "r"((IDT.raw.len() * 8 - 1) << 48)
        );
    }

    pub unsafe fn register(vec: InterruptVector, offset: VirtualAddress) {
        let pair = DescriptorEntry::gate_descriptor(
            offset,
            KERNEL_CODE,
            PrivilegeLevel::Kernel,
            DescriptorType::InterruptGate,
        );
        let table_offset = vec.0 as usize * 2;
        IDT.raw[table_offset + 1] = pair.high;
        IDT.raw[table_offset] = pair.low;
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Msr {
    Tsc = 0x10,
    ApicBase = 0x01b,
    MiscEnable = 0x1a0,
    TscDeadline = 0x6e0,
    Efer = 0xc000_0080,
    Star = 0xc000_0081,
    LStar = 0xc000_0082,
    CStr = 0xc000_0083,
    Fmask = 0xc000_0084,
    FsBase = 0xc000_0100,
    GsBase = 0xc000_0101,
    KernelGsBase = 0xc000_0102,
    TscAux = 0xc000_0103,
    Deadbeef = 0xdeadbeef,
}

#[repr(C)]
#[derive(Copy, Clone)]
union MsrResult {
    pub qword: u64,
    pub tuple: EaxEdx,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct EaxEdx {
    pub eax: u32,
    pub edx: u32,
}

impl Msr {
    pub unsafe fn write(&self, value: u64) {
        let value = MsrResult { qword: value };
        llvm_asm!("wrmsr"
        :: "{eax}"(value.tuple.eax),"{edx}"(value.tuple.edx),"{ecx}"(*self));
    }

    pub unsafe fn read(&self) -> u64 {
        let mut eax: u32;
        let mut edx: u32;
        llvm_asm!("rdmsr"
        : "={eax}"(eax),"={edx}"(edx)
        : "{ecx}"(*self));
        MsrResult {
            tuple: EaxEdx { eax: eax, edx: edx },
        }
        .qword
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct ExceptionStackFrame {
    pub rip: VirtualAddress,
    pub cs: u64,
    pub flags: u64,
    pub rsp: VirtualAddress,
    pub ss: u64,
}

extern "x86-interrupt" fn interrupt_ud_handler(stack_frame: &ExceptionStackFrame) {
    panic!("INVALID OPCODE {:?}", stack_frame,);
}

extern "x86-interrupt" fn interrupt_df_handler(
    stack_frame: &ExceptionStackFrame,
    _error_code: u64,
) {
    panic!("DOUBLE FAULT {:?}", stack_frame,);
}

extern "x86-interrupt" fn interrupt_gp_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    panic!(
        "GENERAL PROTECTION FAULT {:04x} {:?}",
        error_code, stack_frame,
    );
}

extern "x86-interrupt" fn interrupt_page_handler(
    stack_frame: &ExceptionStackFrame,
    error_code: u64,
) {
    let mut cr2: u64;
    unsafe {
        llvm_asm!("mov %cr2, $0":"=r"(cr2));
    }
    panic!(
        "PAGE FAULT {:04x} {:016x} {:?}",
        error_code, cr2, stack_frame,
    );
}