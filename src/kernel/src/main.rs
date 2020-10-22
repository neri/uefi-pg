// My OS Entry
// (c) 2020 Nerry
// License: MIT

#![no_std]
#![no_main]
#![feature(asm)]

// use acpi;
use alloc::boxed::Box;
use alloc::vec::*;
use bootprot::*;
use core::fmt::Write;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, RawWaker, RawWakerVTable, Waker};
use core::time::Duration;
use io::fonts::*;
use io::graphics::*;
use kernel::*;
use mem::memory::*;
use mem::string;
use system::*;
use task::scheduler::*;
use window::view::*;
use window::*;

extern crate alloc;
extern crate rlibc;

// use expr::simple_executor::*;
// use expr::*;
// use futures_util::stream::StreamExt;

myos_entry!(main);

fn main() {
    let mut _main_window: Option<WindowHandle> = None;
    if System::is_headless() {
        stdout().reset().unwrap();
    } else {
        {
            // Main Terminal
            let (console, window) =
                GraphicalConsole::new("Terminal", (80, 24), FontManager::fixed_system_font(), 0, 0);
            window.move_to(Point::new(16, 40));
            window.set_active();
            System::set_stdout(console);
            _main_window = Some(window);
        }

        if false {
            // Test Window 1
            let window = WindowBuilder::new("Welcome")
                .size(Size::new(512, 384))
                .center()
                .build();

            if let Some(view) = window.view() {
                let mut rect = view.bounds();
                rect.size.height = 56;
                let mut shape = View::with_frame(rect);
                // shape.set_background_color(Color::from_rgb(0x64B5F6));
                shape.set_background_color(Color::from_rgb(0x2196F3));
                // shape.set_background_color(Color::from_rgb(0xFF9800));
                view.add_subview(shape);

                let mut rect = view.bounds().insets_by(EdgeInsets::new(16, 16, 0, 16));
                rect.size.height = 44;
                let mut text_view = TextView::with_text("Welcome to My OS !");
                FontDescriptor::new(FontFamily::SansSerif, 32).map(|font| text_view.set_font(font));
                text_view.set_tint_color(IndexedColor::White.into());
                text_view.set_frame(rect);
                text_view.set_max_lines(1);
                view.add_subview(text_view);

                // rect.origin.y += rect.size.height + 10;
                // rect.size.height = 24;
                // let mut text_view = TextView::with_text("~ A toy that displays a picture ~");
                // FontDescriptor::new(FontFamily::Cursive, 20).map(|font| text_view.set_font(font));
                // text_view.set_tint_color(IndexedColor::Green.into());
                // text_view.set_frame(rect);
                // text_view.set_max_lines(2);
                // view.add_subview(text_view);

                rect.origin.y += rect.size.height + 10;
                let mut text_view = TextView::with_text("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.");
                // let mut text_view = TextView::with_text("The quick brown fox jumps over the lazy dog.");
                text_view.set_frame(rect);
                FontDescriptor::new(FontFamily::Serif, 24).map(|font| text_view.set_font(font));
                text_view.set_tint_color(IndexedColor::DarkGray.into());
                text_view.set_max_lines(2);
                text_view.set_bounds(
                    text_view
                        .size_that_fits(Size::new(rect.width(), isize::MAX))
                        .into(),
                );
                view.add_subview(text_view);

                rect.origin.y += rect.size.height + 10;
                let mut text_view = TextView::with_text("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.");
                text_view.set_frame(rect);
                FontDescriptor::new(FontFamily::SansSerif, 20).map(|font| text_view.set_font(font));
                text_view.set_tint_color(IndexedColor::DarkGray.into());
                text_view.set_max_lines(2);
                text_view.set_bounds(
                    text_view
                        .size_that_fits(Size::new(rect.width(), isize::MAX))
                        .into(),
                );
                view.add_subview(text_view);

                rect.origin.y += rect.size.height + 10;
                let mut text_view = TextView::with_text("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.");
                text_view.set_frame(rect);
                FontDescriptor::new(FontFamily::Cursive, 16).map(|font| text_view.set_font(font));
                text_view.set_tint_color(IndexedColor::DarkGray.into());
                text_view.set_max_lines(2);
                text_view.set_bounds(
                    text_view
                        .size_that_fits(Size::new(rect.width(), isize::MAX))
                        .into(),
                );
                view.add_subview(text_view);

                let vertical_base = Coordinates::from_rect(rect).unwrap().bottom + 20;

                let mut button = Button::new(ButtonType::Default);
                button.set_title("OK");
                button.set_frame(Rect::new(10, vertical_base, 120, 30));
                view.add_subview(button);

                let mut button = Button::new(ButtonType::Normal);
                button.set_title("Cancel");
                button.set_frame(Rect::new(140, vertical_base, 120, 30));
                view.add_subview(button);

                let mut button = Button::new(ButtonType::Destructive);
                button.set_title("Destructive");
                button.set_frame(Rect::new(270, vertical_base, 120, 30));
                view.add_subview(button);
            }

            window.set_active();
        }
    }

    if false {
        // It's a joke
        let window = WindowBuilder::new("")
            .style(WindowStyle::NAKED | WindowStyle::FLOATING)
            .frame(Rect::new(-168, 32, 160, 48))
            .bg_color(Color::TRANSPARENT)
            .build();
        window
            .draw(|bitmap| {
                let raduis = 10;
                bitmap.fill_round_rect(bitmap.bounds(), raduis, Color::from_argb(0xE0607D8B));
                bitmap.draw_round_rect(bitmap.bounds(), raduis, IndexedColor::White.into());
                AttributedString::with(
                    "Connected:\n PCI Card",
                    FontManager::label_font(),
                    IndexedColor::White.into(),
                )
                .draw(
                    bitmap,
                    bitmap.bounds().insets_by(EdgeInsets::padding_all(8)),
                );
            })
            .unwrap();
        window.show();

        let window = WindowBuilder::new("")
            .style(WindowStyle::NAKED | WindowStyle::FLOATING)
            .frame(Rect::new(-168, 88, 160, 48))
            .bg_color(Color::TRANSPARENT)
            .build();
        window
            .draw(|bitmap| {
                let raduis = 10;
                bitmap.fill_round_rect(bitmap.bounds(), raduis, Color::from_argb(0xE0607D8B));
                bitmap.draw_round_rect(bitmap.bounds(), raduis, IndexedColor::White.into());
                AttributedString::with(
                    "Connected:\n USB Device",
                    FontManager::label_font(),
                    IndexedColor::White.into(),
                )
                .draw(
                    bitmap,
                    bitmap.bounds().insets_by(EdgeInsets::padding_all(8)),
                );
            })
            .unwrap();
        window.show();
    }

    let mut tasks: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();

    if System::is_headless() {
    } else {
        tasks.push(Box::pin(status_bar_main()));
        tasks.push(Box::pin(activity_monitor_main()));
    }
    tasks.push(Box::pin(repl_main(_main_window)));

    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    loop {
        for task in &mut tasks {
            let _ = task.as_mut().poll(&mut cx);
        }
        Timer::usleep(100_000);
    }
}

#[allow(dead_code)]
async fn repl_main(_main_window: Option<WindowHandle>) {
    exec("ver");

    loop {
        print!("# ");
        if let Some(cmdline) = stdout().read_line_async(120).await {
            exec(&cmdline);
        }
    }
}

fn exec(cmdline: &str) {
    if cmdline.len() == 0 {
        return;
    }
    let mut sb = string::StringBuffer::with_capacity(cmdline.len());
    let mut args = Vec::new();
    let mut phase = CmdLinePhase::LeadingSpace;
    sb.clear();
    for c in cmdline.chars() {
        match phase {
            CmdLinePhase::LeadingSpace => match c {
                ' ' => (),
                _ => {
                    sb.write_char(c).unwrap();
                    phase = CmdLinePhase::Token;
                }
            },
            CmdLinePhase::Token => match c {
                ' ' => {
                    args.push(sb.as_str());
                    phase = CmdLinePhase::LeadingSpace;
                    sb.split();
                }
                _ => {
                    sb.write_char(c).unwrap();
                }
            },
        }
    }
    if sb.len() > 0 {
        args.push(sb.as_str());
    }

    if args.len() > 0 {
        let cmd = args[0];
        match command(cmd) {
            Some(exec) => {
                exec(args.as_slice());
            }
            None => println!("Command not found: {}", cmd),
        }
    }
}

enum CmdLinePhase {
    LeadingSpace,
    Token,
}

fn command(cmd: &str) -> Option<&'static fn(&[&str]) -> isize> {
    for command in &COMMAND_TABLE {
        if command.0 == cmd {
            return Some(&command.1);
        }
    }
    None
}

const COMMAND_TABLE: [(&str, fn(&[&str]) -> isize, &str); 8] = [
    ("help", cmd_help, "Show Help"),
    ("cls", cmd_cls, "Clear screen"),
    ("ver", cmd_ver, "Display version"),
    ("sysctl", cmd_sysctl, "System Control"),
    ("lspci", cmd_lspci, "Show List of PCI Devices"),
    ("reboot", cmd_reboot, "Restart computer"),
    ("exit", cmd_reserved, ""),
    ("echo", cmd_echo, ""),
];

fn cmd_reserved(_: &[&str]) -> isize {
    println!("Feature not available");
    1
}

fn cmd_reboot(_: &[&str]) -> isize {
    unsafe {
        System::reset();
    }
}

fn cmd_help(_: &[&str]) -> isize {
    for cmd in &COMMAND_TABLE {
        if cmd.2.len() > 0 {
            println!("{}\t{}", cmd.0, cmd.2);
        }
    }
    0
}

fn cmd_cls(_: &[&str]) -> isize {
    match stdout().reset() {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

fn cmd_ver(_: &[&str]) -> isize {
    println!("{} v{}", System::name(), System::version(),);
    0
}

fn cmd_echo(args: &[&str]) -> isize {
    println!("{}", args[1..].join(" "));
    0
}

use arch::cpu::*;
fn cmd_sysctl(argv: &[&str]) -> isize {
    if argv.len() < 2 {
        println!("usage: sysctl command [options]");
        println!("memory:\tShow memory information");
        return 1;
    }
    let subcmd = argv[1];
    match subcmd {
        "memory" => {
            let mut sb = string::StringBuffer::with_capacity(256);
            MemoryManager::statistics(&mut sb);
            print!("{}", sb.as_str());
        }
        "random" => match Cpu::secure_rand() {
            Ok(rand) => println!("{:016x}", rand),
            Err(_) => println!("# No SecureRandom"),
        },
        "cpuid" => {
            let cpuid0 = Cpu::cpuid(0, 0);
            let cpuid1 = Cpu::cpuid(1, 0);
            let cpuid7 = Cpu::cpuid(7, 0);
            let cpuid81 = Cpu::cpuid(0x8000_0001, 0);
            println!("CPUID {:08x}", cpuid0.eax());
            println!(
                "Feature 0000_0001 EDX {:08x} ECX {:08x}",
                cpuid1.edx(),
                cpuid1.ecx(),
            );
            println!(
                "Feature 0000_0007 EBX {:08x} ECX {:08x} EDX {:08x}",
                cpuid7.ebx(),
                cpuid7.ecx(),
                cpuid7.edx(),
            );
            println!(
                "Feature 8000_0001 EDX {:08x} ECX {:08x}",
                cpuid81.edx(),
                cpuid81.ecx(),
            );
            if cpuid0.eax() >= 0x0B {
                let cpuid0b = Cpu::cpuid(0x0B, 0);
                println!(
                    "CPUID0B: {:08x} {:08x} {:08x} {:08x}",
                    cpuid0b.eax(),
                    cpuid0b.ebx(),
                    cpuid0b.ecx(),
                    cpuid0b.edx()
                );
            }
        }
        _ => {
            println!("Unknown command: {}", subcmd);
            return 1;
        }
    }
    0
}

fn cmd_lspci(_: &[&str]) -> isize {
    for device in bus::pci::Pci::devices() {
        let addr = device.address();
        println!(
            "{:02x}.{:02x}.{} {:04x}:{:04x} {:06x} {}",
            addr.0,
            addr.1,
            addr.2,
            device.vendor_id().0,
            device.device_id().0,
            device.class_code(),
            device.class_string(),
        );
        if false {
            for function in device.functions() {
                let addr = function.address();
                println!(
                    "     .{} {:04x}:{:04x} {:06x} {}",
                    addr.2,
                    function.vendor_id().0,
                    function.device_id().0,
                    function.class_code(),
                    function.class_string(),
                );
            }
        }
    }
    0
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

#[allow(dead_code)]
async fn status_bar_main() {
    const STATUS_BAR_HEIGHT: isize = 24;
    let bg_color = Color::from_argb(0xC0EEEEEE);
    let fg_color = IndexedColor::DarkGray.into();

    let screen_bounds = WindowManager::main_screen_bounds();
    let window = WindowBuilder::new("Status Bar")
        .style(WindowStyle::NAKED | WindowStyle::FLOATING)
        .style_add(WindowStyle::BORDER)
        .frame(Rect::new(0, 0, screen_bounds.width(), STATUS_BAR_HEIGHT))
        .bg_color(bg_color)
        .build();

    let mut ats = AttributedString::with("My OS", FontManager::title_font(), fg_color);

    window
        .draw(|bitmap| {
            let bounds = bitmap.bounds();
            let size = ats.bounding_size(Size::new(isize::MAX, isize::MAX));
            let rect = Rect::new(
                16,
                (bounds.height() - size.height) / 2,
                size.width,
                size.height,
            );
            ats.draw(&bitmap, rect);
        })
        .unwrap();
    window.show();
    WindowManager::add_screen_insets(EdgeInsets::new(STATUS_BAR_HEIGHT, 0, 0, 0));

    ats.set_font(FontManager::system_font());
    let mut sb = string::Sb255::new();
    loop {
        Timer::sleep_async(Duration::from_millis(500)).await;

        sb.clear();

        // let usage = MyScheduler::usage_per_cpu();
        // let usage0 = usage / 10;
        // let usage1 = usage % 10;
        // write!(sb, "{:3}.{:1}%  ", usage0, usage1).unwrap();

        let time = System::system_time();
        let tod = time.secs % 86400;
        let sec = tod % 60;
        let min = tod / 60 % 60;
        let hour = tod / 3600;
        if sec % 2 == 0 {
            write!(sb, "{:2} {:02} {:02}", hour, min, sec).unwrap();
        } else {
            write!(sb, "{:2}:{:02}:{:02}", hour, min, sec).unwrap();
        };
        ats.set_text(sb.as_str());

        let bounds = window.frame();
        let width = ats.bounding_size(Size::new(isize::MAX, isize::MAX)).width;
        let rect = Rect::new(
            bounds.width() - width - 16,
            (bounds.height() - ats.font().line_height()) / 2,
            width,
            ats.font().line_height(),
        );
        let _ = window.draw(|bitmap| {
            bitmap.fill_rect(rect, bg_color);
            ats.draw(&bitmap, rect);
        });
    }
}

#[allow(dead_code)]
async fn activity_monitor_main() {
    let bg_color = Color::from(IndexedColor::Black).set_opacity(0xC0);
    let fg_color = IndexedColor::Yellow.into();
    let graph_sub_color = IndexedColor::LightGreen.into();
    let graph_main_color = IndexedColor::Yellow.into();
    let graph_border_color = IndexedColor::LightGray.into();

    // Timer::sleep_async(Duration::from_millis(2000)).await;

    let window = WindowBuilder::new("Activity Monitor")
        .style_add(WindowStyle::NAKED | WindowStyle::FLOATING | WindowStyle::PINCHABLE)
        .frame(Rect::new(-330, -180, 320, 150))
        .bg_color(bg_color)
        .build();

    window.show();

    let mut ats = AttributedString::new("");
    FontDescriptor::new(FontFamily::SmallFixed, 8).map(|font| ats.set_font(font));
    ats.set_color(fg_color);

    let num_of_cpus = System::num_of_cpus();
    let n_items = 64;
    let mut usage_temp = Vec::with_capacity(num_of_cpus);
    let mut usage_cursor = 0;
    let mut usage_history = {
        let count = num_of_cpus * n_items;
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(u8::MAX);
        }
        vec
    };

    let mut sb = string::StringBuffer::with_capacity(0x1000);
    let mut time0 = Timer::measure();
    let mut tsc0 = unsafe { Cpu::read_tsc() };
    loop {
        Timer::sleep_async(Duration::from_millis(1000)).await;
        let time1 = Timer::measure();
        let tsc1 = unsafe { Cpu::read_tsc() };
        let hertz = (tsc1 - tsc0) / (time1 - time0);
        let hertz0 = hertz % 1000;
        let hertz1 = hertz / 1000;

        MyScheduler::get_idle_statistics(&mut usage_temp);
        for i in 0..num_of_cpus {
            usage_history[i * n_items + usage_cursor] =
                (u32::min(usage_temp[i], 999) * 254 / 999) as u8;
        }
        usage_cursor = (usage_cursor + 1) % n_items;

        window
            .draw(|bitmap| {
                bitmap.fill_rect(bitmap.bounds(), bg_color);
                for cpu_index in 0..num_of_cpus {
                    let padding = 4;
                    let item_size = Size::new(
                        isize::min(
                            isize::max(
                                (bitmap.bounds().width() - padding) / num_of_cpus as isize
                                    - padding,
                                16,
                            ),
                            n_items as isize,
                        ),
                        32,
                    );
                    let rect = Rect::new(
                        padding + cpu_index as isize * (item_size.width + padding),
                        padding,
                        item_size.width,
                        item_size.height,
                    );
                    let h_lines = 4;
                    let v_lines = 4;
                    for i in 1..h_lines {
                        let point = Point::new(rect.x(), rect.y() + i * item_size.height / h_lines);
                        bitmap.draw_hline(point, item_size.width, graph_sub_color);
                    }
                    for i in 1..v_lines {
                        let point = Point::new(rect.x() + i * item_size.width / v_lines, rect.y());
                        bitmap.draw_vline(point, item_size.height, graph_sub_color);
                    }

                    let limit = item_size.width as usize - 2;
                    for i in 0..limit {
                        let scale = item_size.height - 2;
                        let value1 = usage_history
                            [cpu_index * n_items + ((usage_cursor + i - limit) % n_items)]
                            as isize
                            * scale
                            / 255;
                        let value2 = usage_history
                            [cpu_index * n_items + ((usage_cursor + i - 1 - limit) % n_items)]
                            as isize
                            * scale
                            / 255;
                        let c0 = Point::new(rect.x() + i as isize + 1, rect.y() + 1 + value1);
                        let c1 = Point::new(rect.x() + i as isize, rect.y() + 1 + value2);
                        bitmap.draw_line(c0, c1, graph_main_color);
                    }
                    bitmap.draw_rect(rect, graph_border_color);
                }

                sb.clear();
                let usage = MyScheduler::usage_per_cpu();
                let usage0 = usage % 10;
                let usage1 = usage / 10;
                write!(
                    sb,
                    "CPU: {}.{:03} GHz {:3}.{}%",
                    hertz1, hertz0, usage1, usage0,
                )
                .unwrap();
                let rect = bitmap.bounds().insets_by(EdgeInsets::new(38, 4, 4, 4));
                ats.set_text(sb.as_str());
                ats.draw(&bitmap, rect);

                MyScheduler::print_statistics(&mut sb, true);
                let rect = bitmap.bounds().insets_by(EdgeInsets::new(48, 4, 4, 4));
                ats.set_text(sb.as_str());
                ats.draw(&bitmap, rect);
            })
            .unwrap();

        tsc0 = tsc1;
        time0 = time1;
    }
}
