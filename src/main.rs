// My UEFI-Rust Playground
#![feature(abi_efiapi)]
#![no_std]
#![no_main]
use core::fmt::Write;
use uefi::prelude::*;
use uefi_pg::myos::io::graphics::*;
use uefi_pg::*;

uefi_pg_entry!(main);

fn main(handle: Handle, st: SystemTable<Boot>) -> Status {
    let rsdptr = match st.find_config_table(uefi::table::cfg::ACPI2_GUID) {
        Some(val) => val,
        None => {
            writeln!(st.stdout(), "Error: ACPI Table Not Found").unwrap();
            return Status::LOAD_ERROR;
        }
    };

    // TODO: init custom allocator
    let buf_size = 0x1000000;
    let buf_ptr = st
        .boot_services()
        .allocate_pool(uefi::table::boot::MemoryType::LOADER_DATA, buf_size)
        .unwrap()
        .unwrap();
    myos::mem::alloc::init(buf_ptr as usize, buf_size);

    //////// GUARD //////// exit_boot_services //////// GUARD ////////
    let (_st, mm) = exit_boot_services(st, handle);

    let mut total_memory_size: u64 = 0;
    for mem_desc in mm {
        if mem_desc.ty.is_countable() {
            total_memory_size += mem_desc.page_count << 12;
        }
    }
    unsafe {
        myos::arch::system::System::init(rsdptr as usize, total_memory_size);
    }

    let fb = stdout().fb();
    // fb.reset();
    fb.fill_rect(
        Rect::new(50, 50, 200, 200),
        IndexedColor::LightRed.as_color(),
    );
    fb.fill_rect(
        Rect::new(100, 100, 200, 200),
        IndexedColor::LightGreen.as_color(),
    );
    fb.fill_rect(
        Rect::new(150, 150, 200, 200),
        IndexedColor::LightBlue.as_color(),
    );

    let system = myos::arch::system::System::shared();

    println!(
        "My practice OS version {} Total {} / {} CPU Cores, {} MB System Memory",
        myos::MyOs::version(),
        system.number_of_active_cpus(),
        system.number_of_cpus(),
        system.total_memory_size() >> 20,
    );
    println!("Hello, {:#}!", "Rust");

    loop {
        unsafe {
            myos::arch::cpu::Cpu::halt();
        }
    }
    panic!("System has halted");
    // Status::SUCCESS
}
