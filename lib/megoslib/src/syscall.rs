// MEG-OS System Calls

use megosabi::svc::Function;

#[link(wasm_import_module = "megos-canary")]
extern "C" {
    pub fn svc0(_: Function) -> usize;
    pub fn svc1(_: Function, _: usize) -> usize;
    pub fn svc2(_: Function, _: usize, _: usize) -> usize;
    pub fn svc3(_: Function, _: usize, _: usize, _: usize) -> usize;
    pub fn svc4(_: Function, _: usize, _: usize, _: usize, _: usize) -> usize;
    pub fn svc5(_: Function, _: usize, _: usize, _: usize, _: usize, _: usize) -> usize;
    pub fn svc6(_: Function, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize) -> usize;
}

#[inline]
pub fn os_exit() -> ! {
    unsafe {
        svc0(Function::Exit);
        loop {
            asm!("unreachable");
        }
    }
}

/// Display a string.
#[inline]
pub fn os_print(s: &str) {
    unsafe { svc2(Function::PrintString, s.as_ptr() as usize, s.len()) };
}

/// Get the value of the monotonic timer in microseconds.
#[inline]
pub fn os_monotonic() -> u32 {
    unsafe { svc0(Function::Monotonic) as u32 }
}

#[inline]
pub fn os_bench<F>(f: F) -> usize
where
    F: FnOnce() -> (),
{
    let time0 = unsafe { svc0(Function::Monotonic) };
    f();
    let time1 = unsafe { svc0(Function::Monotonic) };
    time1 - time0
}

#[inline]
pub fn os_time_of_day() -> u32 {
    unsafe { svc1(Function::Time, 0) as u32 }
}

/// Blocks a thread for the specified microseconds.
#[inline]
pub fn os_usleep(us: u32) {
    unsafe { svc1(Function::Usleep, us as usize) };
}

/// Get the system version information.
#[inline]
pub fn os_version() -> u32 {
    unsafe { svc1(Function::GetSystemInfo, 0) as u32 }
}

/// Create a new window.
#[inline]
#[rustfmt::skip]
pub fn os_new_window1(title: &str, width: usize, height: usize) -> usize {
    unsafe { svc4(Function::NewWindow, title.as_ptr() as usize, title.len(), width, height) }
}

/// Create a new window.
#[inline]
#[rustfmt::skip]
pub fn os_new_window2(title: &str, width: usize, height: usize, bg_color: usize, flag: usize) -> usize {
    unsafe { svc6( Function::NewWindow, title.as_ptr() as usize, title.len(), width, height, bg_color, flag) }
}

/// Close a window.
#[inline]
pub fn os_close_window(window: usize) {
    unsafe { svc1(Function::CloseWindow, window) };
}

/// Create a drawing context
#[inline]
pub fn os_begin_draw(window: usize) -> usize {
    unsafe { svc1(Function::BeginDraw, window) }
}

/// Discard the drawing context and reflect it to the screen
#[inline]
pub fn os_end_draw(ctx: usize) {
    unsafe { svc1(Function::EndDraw, ctx) };
}

/// Draw a string in a window.
#[inline]
pub fn os_win_draw_string(ctx: usize, x: usize, y: usize, s: &str, color: usize) {
    let ptr = s.as_ptr() as usize;
    unsafe { svc6(Function::DrawString, ctx, x, y, ptr, s.len(), color) };
}

/// Fill a rectangle in a window.
#[inline]
#[rustfmt::skip]
pub fn os_win_fill_rect(ctx: usize, x: usize, y: usize, width: usize, height: usize, color: usize) {
    unsafe { svc6(Function::FillRect, ctx, x, y, width, height, color) };
}

#[inline]
pub fn os_win_draw_line(ctx: usize, x1: usize, y1: usize, x2: usize, y2: usize, color: usize) {
    unsafe { svc6(Function::DrawLine, ctx, x1, y1, x2, y2, color) };
}

/// Wait for key event
#[inline]
pub fn os_wait_char(window: usize) -> u32 {
    unsafe { svc1(Function::WaitChar, window) as u32 }
}

/// Read a key event
#[inline]
pub fn os_read_char(window: usize) -> u32 {
    unsafe { svc1(Function::ReadChar, window) as u32 }
}

/// Draw a bitmap in a window
#[inline]
pub fn os_blt8(ctx: usize, x: usize, y: usize, bitmap: usize) {
    unsafe { svc4(Function::Blt8, ctx, x, y, bitmap) };
}

#[inline]
pub fn os_blt32(ctx: usize, x: usize, y: usize, bitmap: usize) {
    unsafe { svc4(Function::Blt32, ctx, x, y, bitmap) };
}

/// Draw a bitmap in a window
#[inline]
pub fn os_blt1(ctx: usize, x: usize, y: usize, bitmap: usize, color: u32, mode: usize) {
    unsafe { svc6(Function::Blt1, ctx, x, y, bitmap, color as usize, mode) };
}

/// TEST
#[inline]
#[rustfmt::skip]
pub fn os_blend_rect(bitmap: usize, x: usize, y: usize, width: usize, height: usize, color: u32) {
    unsafe { svc6(Function::BlendRect, bitmap, x, y, width, height, color as usize) };
}

/// Return a random number
#[inline]
pub fn os_rand() -> u32 {
    unsafe { svc0(Function::Rand) as u32 }
}

/// Set the seed of the random number.
#[inline]
pub fn os_srand(srand: u32) -> u32 {
    unsafe { svc1(Function::Srand, srand as usize) as u32 }
}

/// Allocates memory blocks with a simple allocator
#[inline]
pub fn os_alloc(size: usize, align: usize) -> usize {
    unsafe { svc2(Function::Alloc, size, align) }
}

/// Frees an allocated memory block
#[inline]
pub fn os_dealloc(ptr: usize, size: usize, align: usize) {
    unsafe { svc3(Function::Dealloc, ptr, size, align) };
}
