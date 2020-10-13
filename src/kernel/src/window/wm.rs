// A Window System

use super::view::*;
use crate::io::fonts::*;
use crate::io::graphics::*;
use crate::io::hid::*;
use crate::num::*;
use crate::sync::semaphore::*;
use crate::task::scheduler::*;
use crate::*;
use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::vec::*;
use bitflags::*;
use core::cmp;
use core::isize;
use core::num::*;
use core::sync::atomic::*;
use core::time::Duration;

// const MAX_WINDOWS: usize = 256;
const WINDOW_TITLE_LENGTH: usize = 32;

const WINDOW_BORDER_PADDING: isize = 0;
const WINDOW_BORDER_SHADOW_PADDING: isize = 8;
const WINDOW_TITLE_HEIGHT: isize = 24;

const DESKTOP_COLOR: Color = Color::from_argb(0xFF2196F3);
const BARRIER_COLOR: Color = Color::from_argb(0x80000000);
const WINDOW_ACTIVE_TITLE_BG_COLOR: Color = Color::from_argb(0xF0E0E0E0);
const WINDOW_ACTIVE_TITLE_SHADOW_COLOR: Color = Color::from_argb(0xFF9E9E9E);
const WINDOW_ACTIVE_TITLE_FG_COLOR: Color = Color::from_argb(0xFF212121);
const WINDOW_INACTIVE_TITLE_BG_COLOR: Color = Color::from_argb(0xFFEEEEEE);
const WINDOW_INACTIVE_TITLE_FG_COLOR: Color = Color::from_argb(0xFF9E9E9E);

// Mouse Pointer
const MOUSE_POINTER_WIDTH: usize = 12;
const MOUSE_POINTER_HEIGHT: usize = 20;
const MOUSE_POINTER_PALETTE: [u32; 4] = [0x00FF00FF, 0xFFFFFFFF, 0xFF999999, 0xFF000000];
const MOUSE_POINTER_SOURCE: [[u8; MOUSE_POINTER_WIDTH]; MOUSE_POINTER_HEIGHT] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 3, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0],
    [1, 3, 3, 3, 2, 1, 0, 0, 0, 0, 0, 0],
    [1, 3, 3, 3, 3, 2, 1, 0, 0, 0, 0, 0],
    [1, 3, 3, 3, 3, 3, 2, 1, 0, 0, 0, 0],
    [1, 3, 3, 3, 3, 3, 3, 2, 1, 0, 0, 0],
    [1, 3, 3, 3, 3, 3, 3, 3, 2, 1, 0, 0],
    [1, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1, 0],
    [1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1],
    [1, 3, 3, 3, 3, 3, 2, 1, 1, 1, 1, 1],
    [1, 3, 3, 2, 1, 2, 3, 1, 0, 0, 0, 0],
    [1, 3, 2, 1, 0, 1, 3, 2, 1, 0, 0, 0],
    [1, 2, 1, 0, 0, 1, 2, 3, 1, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 1, 3, 2, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 2, 3, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

// Close button
const CLOSE_BUTTON_SIZE: usize = 10;
const CLOSE_BUTTON_PALETTE: [u32; 4] = [0x00000000, 0x40000000, 0x80000000, 0xC0000000];
const CLOSE_BUTTON_SOURCE: [[u8; CLOSE_BUTTON_SIZE]; CLOSE_BUTTON_SIZE] = [
    [0, 1, 0, 0, 0, 0, 0, 0, 1, 0],
    [1, 3, 2, 0, 0, 0, 0, 2, 3, 1],
    [0, 2, 3, 2, 0, 0, 2, 3, 2, 0],
    [0, 0, 2, 3, 2, 2, 3, 2, 0, 0],
    [0, 0, 0, 2, 3, 3, 2, 0, 0, 0],
    [0, 0, 0, 2, 3, 3, 2, 0, 0, 0],
    [0, 0, 2, 3, 2, 2, 3, 2, 0, 0],
    [0, 2, 3, 2, 0, 0, 2, 3, 2, 0],
    [1, 3, 2, 0, 0, 0, 0, 2, 3, 1],
    [0, 1, 0, 0, 0, 0, 0, 0, 1, 0],
];

static mut WM: Option<Box<WindowManager>> = None;

pub struct WindowManager {
    lock: Spinlock,
    sem_redraw: Semaphore,
    attributes: WindowManagerAttributes,
    pointer_x: AtomicIsize,
    pointer_y: AtomicIsize,
    buttons: AtomicUsize,
    button_pressed: AtomicUsize,
    main_screen: &'static Bitmap,
    off_screen: Box<Bitmap>,
    screen_insets: EdgeInsets<isize>,
    resources: Resources,
    pool: BTreeMap<WindowHandle, Box<RawWindow>>,
    pool_lock: Spinlock,
    root: Option<WindowHandle>,
    pointer: Option<WindowHandle>,
    barrier: Option<WindowHandle>,
    active: Option<WindowHandle>,
    captured: Option<WindowHandle>,
    captured_origin: Point<isize>,
}

struct Resources {
    close_button: Bitmap,
    corner_shadow: Bitmap,
}

impl WindowManager {
    pub(crate) fn init(main_screen: &'static Box<Bitmap>) {
        let off_screen = Box::new(Bitmap::with_same_size(main_screen));

        let close_button = {
            let width = CLOSE_BUTTON_SIZE;
            let height = CLOSE_BUTTON_SIZE;
            let mut vec = Vec::with_capacity(width * height);
            for y in 0..height {
                for x in 0..width {
                    let argb = CLOSE_BUTTON_PALETTE[CLOSE_BUTTON_SOURCE[y][x] as usize];
                    vec.push(Color::from_argb(argb));
                }
            }
            Bitmap::from_vec(vec, width, height, true)
        };

        let corner_shadow = {
            let w = WINDOW_BORDER_SHADOW_PADDING;
            let h = WINDOW_BORDER_SHADOW_PADDING;
            let bitmap = Bitmap::new(w as usize * 2, h as usize * 2, false);
            bitmap.reset();
            let center = bitmap.bounds().center();
            for q in 0..WINDOW_BORDER_SHADOW_PADDING {
                let r = WINDOW_BORDER_SHADOW_PADDING - q;
                let density = ((q + 1) * (q + 1)) as u8;
                bitmap.fill_circle(center, r, Color::gray(0, density));
            }
            bitmap
        };

        let wm = Some(Box::new(WindowManager {
            lock: Spinlock::default(),
            sem_redraw: Semaphore::new(0),
            attributes: WindowManagerAttributes::EMPTY,
            pointer_x: AtomicIsize::new(main_screen.width() / 2),
            pointer_y: AtomicIsize::new(main_screen.height() / 2),
            buttons: AtomicUsize::new(0),
            button_pressed: AtomicUsize::new(0),
            main_screen,
            off_screen,
            screen_insets: EdgeInsets::zero(),
            resources: Resources {
                close_button,
                corner_shadow,
                // font: FontDriver::system_font(),
            },
            pool: BTreeMap::new(),
            pool_lock: Spinlock::default(),
            root: None,
            pointer: None,
            barrier: None,
            active: None,
            captured: None,
            captured_origin: Point::zero(),
        }));
        unsafe {
            WM = wm;
        }
        let shared = Self::shared();

        {
            // Root Window (Desktop)
            shared.root = Some(
                WindowBuilder::new("Desktop")
                    .style(WindowStyle::NAKED)
                    .level(WindowLevel::ROOT)
                    .frame(Rect::from(main_screen.size()))
                    .bg_color(DESKTOP_COLOR)
                    .no_bitmap()
                    .build(),
            );
        }

        {
            // Pointer
            let w = MOUSE_POINTER_WIDTH;
            let h = MOUSE_POINTER_HEIGHT;
            let pointer = WindowBuilder::new("Pointer")
                .style(WindowStyle::NAKED)
                .level(WindowLevel::POINTER)
                .origin(shared.pointer())
                .size(Size::new(w as isize, h as isize))
                .bg_color(Color::from_argb(0x80FF00FF))
                .build();

            pointer
                .draw(|bitmap| {
                    for y in 0..h {
                        for x in 0..w {
                            let c = Color::from_argb(
                                MOUSE_POINTER_PALETTE[MOUSE_POINTER_SOURCE[y][x] as usize],
                            );
                            bitmap.draw_pixel(Point::new(x as isize, y as isize), c);
                        }
                    }
                })
                .unwrap();

            pointer.show();
            shared.pointer = Some(pointer);
        }

        {
            // Popup Window Barrier
            shared.barrier = Some(
                WindowBuilder::new("Barrier")
                    .style(WindowStyle::NAKED | WindowStyle::TRANSPARENT)
                    .level(WindowLevel::POPUP_BARRIER)
                    .frame(Rect::from(main_screen.size()))
                    .bg_color(BARRIER_COLOR)
                    .no_bitmap()
                    .build(),
            );
        }

        // If uncomment, you can use the wallpaper (experimental)
        // {
        //     let data = include_bytes!("wall.bmp");
        //     let wallpaper = Bitmap::from_msdib(data).unwrap();
        //     let window = WindowBuilder::new("wall")
        //         .style(WindowStyle::NAKED)
        //         .level(WindowLevel::DESKTOP_ITEMS)
        //         .frame(wallpaper.size().into())
        //         .build();
        //     window
        //         .draw(|bitmap| {
        //             bitmap.blt(
        //                 &wallpaper,
        //                 Point::zero(),
        //                 wallpaper.size().into(),
        //                 BltOption::empty(),
        //             );
        //         })
        //         .unwrap();
        //     window.show();
        // }

        SpawnOption::with_priority(Priority::High)
            .new_pid()
            .spawn_f(Self::winmgr_thread, 0, "Window Manager");
    }

    /// Window Manager Thread
    fn winmgr_thread(_: usize) {
        let shared = WindowManager::shared();
        shared.root.unwrap().invalidate();
        loop {
            let _ = shared.sem_redraw.wait(Duration::from_millis(1000));

            if shared
                .attributes
                .test_and_clear(WindowManagerAttributes::MOUSE_MOVE)
            {
                let origin = shared.pointer();
                let current_button =
                    MouseButton::from_bits_truncate(shared.buttons.load(Ordering::Acquire) as u8);
                let button_pressed = MouseButton::from_bits_truncate(
                    shared.button_pressed.swap(0, Ordering::AcqRel) as u8,
                );

                if let Some(captured) = shared.captured {
                    if current_button.contains(MouseButton::LEFT) {
                        let top = if captured.as_ref().level < WindowLevel::FLOATING {
                            shared.screen_insets.top
                        } else {
                            0
                        };
                        let x = origin.x - shared.captured_origin.x;
                        let y = cmp::max(origin.y - shared.captured_origin.y, top);
                        captured.move_to(Point::new(x, y));
                    } else {
                        shared.captured = None;
                    }
                } else if button_pressed.contains(MouseButton::LEFT) {
                    let mouse_at = Self::window_at_point(origin);
                    if let Some(active) = shared.active {
                        if active != mouse_at {
                            WindowManager::set_active(Some(mouse_at));
                        }
                    } else {
                        WindowManager::set_active(Some(mouse_at));
                    }
                    let target = mouse_at.as_ref();
                    if target.style.contains(WindowStyle::PINCHABLE) {
                        shared.captured = Some(mouse_at);
                        shared.captured_origin = origin - target.frame().origin;
                    } else {
                        let mut title_frame = target.title_frame();
                        title_frame.origin += target.frame.origin;
                        if title_frame.hit_test_point(origin) {
                            shared.captured = Some(mouse_at);
                            shared.captured_origin = origin - target.frame().origin;
                        }
                    }
                }

                shared.pointer.unwrap().move_to(origin);
            }
        }
    }

    #[inline]
    pub fn is_enabled() -> bool {
        unsafe { WM.is_some() }
    }

    #[inline]
    #[track_caller]
    fn shared() -> &'static mut Self {
        unsafe { WM.as_mut().unwrap() }
    }

    #[inline]
    fn synchronized<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let shared = unsafe { WM.as_ref().unwrap() };
        shared.lock.synchronized(f)
    }

    fn next_window_handle() -> WindowHandle {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
        WindowHandle::new(NEXT_ID.fetch_add(1, Ordering::SeqCst)).unwrap()
    }

    #[inline]
    fn synchronized_pool<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let shared = unsafe { WM.as_ref().unwrap() };
        shared.pool_lock.synchronized(f)
    }

    fn add(window: Box<RawWindow>) {
        WindowManager::synchronized_pool(|| {
            let shared = Self::shared();
            shared.pool.insert(window.handle, window);
        })
    }

    #[inline]
    fn get(&self, key: &WindowHandle) -> Option<&Box<RawWindow>> {
        Self::synchronized_pool(|| self.pool.get(key))
    }

    #[inline]
    fn get_mut(&mut self, key: &WindowHandle) -> Option<&mut Box<RawWindow>> {
        Self::synchronized_pool(move || self.pool.get_mut(key))
    }

    unsafe fn add_hierarchy(window: WindowHandle) {
        Self::remove_hierarchy(window);

        let shared = WindowManager::shared();
        let mut cursor = shared.root.unwrap();
        let level = window.as_ref().level;

        loop {
            if let Some(next) = cursor.as_ref().next {
                if level < next.as_ref().level {
                    cursor.update(|cursor| {
                        cursor.next = Some(window);
                    });
                    window.update(|window| {
                        window.next = Some(next);
                    });
                    break;
                } else {
                    cursor = next;
                }
            } else {
                cursor.update(|cursor| {
                    cursor.next = Some(window);
                });
                break;
            }
        }
        window.as_ref().attributes.insert(WindowAttributes::VISIBLE);
    }

    unsafe fn remove_hierarchy(window: WindowHandle) {
        let shared = WindowManager::shared();
        let mut cursor = shared.root.unwrap();

        window.as_ref().attributes.remove(WindowAttributes::VISIBLE);
        loop {
            if let Some(next) = cursor.as_ref().next {
                if next == window {
                    cursor.update(|cursor| {
                        cursor.next = window.as_ref().next;
                    });
                    window.update(|window| {
                        window.next = None;
                    });
                    break;
                }
                cursor = next;
            } else {
                break;
            }
        }
    }

    #[inline]
    pub fn main_screen_bounds() -> Rect<isize> {
        let shared = Self::shared();
        shared.main_screen.bounds()
    }

    #[inline]
    pub fn screen_insets() -> EdgeInsets<isize> {
        let shared = Self::shared();
        shared.screen_insets
    }

    #[inline]
    pub fn add_screen_insets(insets: EdgeInsets<isize>) {
        let shared = Self::shared();
        shared.screen_insets += insets;
    }

    #[inline]
    pub fn invalidate_screen(rect: Rect<isize>) {
        let shared = Self::shared();
        shared.root.unwrap().invalidate_rect(rect);
    }

    fn set_active(window: Option<WindowHandle>) {
        let shared = Self::shared();
        if let Some(old_active) = shared.active {
            shared.active = window;
            old_active.as_ref().draw_frame();
            old_active.invalidate();
            if let Some(active) = window {
                // active.as_ref().draw_frame();
                active.show();
            }
        } else {
            shared.active = window;
            if let Some(active) = window {
                // active.as_ref().draw_frame();
                active.show();
            }
        }
    }

    fn window_at_point(point: Point<isize>) -> WindowHandle {
        WindowManager::synchronized(|| {
            let shared = Self::shared();
            let mut found = shared.root.unwrap();
            let mut cursor = found;
            loop {
                let window = cursor.as_ref();
                if window.level == WindowLevel::POINTER {
                    break found;
                }
                if window
                    .frame
                    .insets_by(window.shadow_insets)
                    .hit_test_point(point)
                {
                    found = cursor;
                }
                cursor = window.next.unwrap();
            }
        })
    }

    fn pointer(&self) -> Point<isize> {
        Point::new(
            self.pointer_x.load(Ordering::Relaxed),
            self.pointer_y.load(Ordering::Relaxed),
        )
    }

    fn update_coord(
        coord: &AtomicIsize,
        movement: isize,
        min_value: isize,
        max_value: isize,
    ) -> bool {
        match coord.fetch_update(Ordering::Acquire, Ordering::Acquire, |value| {
            let new_value = cmp::min(cmp::max(value + movement, min_value), max_value);
            if value == new_value {
                None
            } else {
                Some(new_value)
            }
        }) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn make_mouse_event(mouse_state: &mut MouseState) {
        if !Self::is_enabled() {
            return;
        }
        let shared = Self::shared();
        let bounds = shared.main_screen.bounds();

        let mut pointer = Point::new(0, 0);
        core::mem::swap(&mut mouse_state.x, &mut pointer.x);
        core::mem::swap(&mut mouse_state.y, &mut pointer.y);
        let button_change = mouse_state.current_buttons ^ mouse_state.prev_buttons;
        let button_pressed = button_change & mouse_state.current_buttons;

        shared.buttons.store(
            mouse_state.current_buttons.bits() as usize,
            Ordering::Release,
        );

        shared
            .button_pressed
            .fetch_or(button_pressed.bits() as usize, Ordering::AcqRel);

        Self::update_coord(&shared.pointer_x, pointer.x, bounds.x(), bounds.width() - 1);
        Self::update_coord(
            &shared.pointer_y,
            pointer.y,
            bounds.y(),
            bounds.height() - 1,
        );
        shared
            .attributes
            .insert(WindowManagerAttributes::MOUSE_MOVE);
        shared.sem_redraw.signal();
    }
}

#[derive(Default)]
struct WindowManagerAttributes(AtomicUsize);

#[allow(dead_code)]
impl WindowManagerAttributes {
    pub const EMPTY: Self = Self::new(0);
    pub const MOUSE_MOVE: usize = 0b0000_0001;

    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(AtomicUsize::new(value))
    }

    #[inline]
    pub fn contains(&self, value: usize) -> bool {
        (self.0.load(Ordering::Acquire) & value) == value
    }

    #[inline]
    pub fn insert(&self, value: usize) {
        self.0.fetch_or(value, Ordering::AcqRel);
    }

    #[inline]
    pub fn remove(&self, value: usize) {
        self.0.fetch_and(!value, Ordering::AcqRel);
    }

    fn test_and_clear(&self, bits: usize) -> bool {
        self.0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
                if (x & bits) == bits {
                    Some(x & !bits)
                } else {
                    None
                }
            })
            .is_ok()
    }
}

#[allow(dead_code)]
struct RawWindow {
    handle: WindowHandle,
    next: Option<WindowHandle>,
    frame: Rect<isize>,
    shadow_insets: EdgeInsets<isize>,
    content_insets: EdgeInsets<isize>,
    attributes: WindowAttributes,
    style: WindowStyle,
    level: WindowLevel,
    bg_color: Color,
    bitmap: Option<Box<Bitmap>>,
    view: Option<Box<dyn ViewTrait>>,
    title: [u8; WINDOW_TITLE_LENGTH],
}

bitflags! {
    pub struct WindowStyle: u8 {
        const BORDER        = 0b0000_0001;
        const TITLE         = 0b0000_0010;
        const NAKED         = 0b0000_0100;
        const TRANSPARENT   = 0b0000_1000;
        const PINCHABLE     = 0b0001_0000;
        const FLOATING      = 0b0010_0000;

        const DEFAULT = Self::TRANSPARENT.bits | Self::BORDER.bits | Self::TITLE.bits;
    }
}

impl WindowStyle {
    fn as_content_insets(self) -> EdgeInsets<isize> {
        let mut insets = if self.contains(Self::BORDER) {
            EdgeInsets::padding_all(WINDOW_BORDER_PADDING)
        } else {
            EdgeInsets::zero()
        };
        if self.contains(Self::TITLE) {
            insets.top += WINDOW_TITLE_HEIGHT;
        }
        insets
    }
}

struct WindowAttributes(AtomicU8);

#[allow(dead_code)]
impl WindowAttributes {
    pub const EMPTY: Self = Self::new(0);
    pub const NEEDS_REDRAW: u8 = 0b0000_0001;
    pub const VISIBLE: u8 = 0b0000_0010;

    #[inline]
    pub const fn new(value: u8) -> Self {
        Self(AtomicU8::new(value))
    }

    #[inline]
    pub fn contains(&self, value: u8) -> bool {
        (self.0.load(Ordering::Acquire) & value) == value
    }

    #[inline]
    pub fn insert(&self, value: u8) {
        self.0.fetch_or(value, Ordering::AcqRel);
    }

    #[inline]
    pub fn remove(&self, value: u8) {
        self.0.fetch_and(!value, Ordering::AcqRel);
    }
}

impl RawWindow {
    // #[inline]
    // fn bounds(&self) -> Rect<isize> {
    //     Rect::from(self.frame.insets_by(self.shadow_insets).size)
    // }

    #[inline]
    fn actual_bounds(&self) -> Rect<isize> {
        self.frame.size().into()
    }

    #[inline]
    fn frame(&self) -> Rect<isize> {
        self.frame.insets_by(self.shadow_insets)
    }

    fn set_frame(&mut self, new_frame: Rect<isize>) {
        let old_frame = self.frame;
        let new_frame = Rect::new(
            new_frame.x() - self.shadow_insets.left,
            new_frame.y() - self.shadow_insets.top,
            new_frame.width() + self.shadow_insets.left + self.shadow_insets.right,
            new_frame.height() + self.shadow_insets.top + self.shadow_insets.bottom,
        );
        if old_frame != new_frame {
            self.frame = new_frame;
            if self.attributes.contains(WindowAttributes::VISIBLE) {
                WindowManager::invalidate_screen(old_frame);
                self.draw_frame();
                self.invalidate();
            }
        }
    }

    fn draw_to_screen(&self, rect: Rect<isize>, is_offscreen: bool) {
        let mut frame = rect;
        frame.origin += self.frame.origin;
        let coords1 = match Coordinates::from_rect(frame) {
            Some(coords) => coords,
            None => return,
        };

        let main_screen = WindowManager::shared().main_screen;
        let off_screen = WindowManager::shared().off_screen.as_ref();
        let target_screen = if is_offscreen {
            off_screen
        } else {
            main_screen
        };

        let mut cursor = WindowManager::shared().root.unwrap();

        loop {
            let window = cursor.as_ref();
            if let Some(coords2) = Coordinates::from_rect(window.frame) {
                if frame.hit_test_rect(window.frame) {
                    let blt_origin = Point::new(
                        cmp::max(coords1.left, coords2.left),
                        cmp::max(coords1.top, coords2.top),
                    );
                    let x = if coords1.left > coords2.left {
                        coords1.left - coords2.left
                    } else {
                        0
                    };
                    let y = if coords1.top > coords2.top {
                        coords1.top - coords2.top
                    } else {
                        0
                    };
                    let blt_rect = Rect::new(
                        x,
                        y,
                        cmp::min(coords1.right, coords2.right)
                            - cmp::max(coords1.left, coords2.left),
                        cmp::min(coords1.bottom, coords2.bottom)
                            - cmp::max(coords1.top, coords2.top),
                    );

                    if let Some(bitmap) = &window.bitmap {
                        target_screen.blt(bitmap, blt_origin, blt_rect, BltOption::empty());
                    } else {
                        if window.style.contains(WindowStyle::TRANSPARENT) {
                            target_screen.blend_rect(blt_rect, window.bg_color);
                        } else {
                            target_screen.fill_rect(blt_rect, window.bg_color);
                        }
                    }
                }
            }
            cursor = match window.next {
                Some(next) => next,
                None => break,
            };
        }
        if is_offscreen {
            main_screen.blt(off_screen, frame.origin, frame, BltOption::empty());
        }
    }

    fn title_frame(&self) -> Rect<isize> {
        if self.style.contains(WindowStyle::TITLE) {
            Rect::new(
                WINDOW_BORDER_SHADOW_PADDING + WINDOW_BORDER_PADDING,
                WINDOW_BORDER_SHADOW_PADDING + WINDOW_BORDER_PADDING,
                self.frame.width() - WINDOW_BORDER_PADDING * 2 - WINDOW_BORDER_SHADOW_PADDING * 2,
                WINDOW_TITLE_HEIGHT,
            )
        } else {
            Rect::zero()
        }
    }

    fn is_active(&self) -> bool {
        WindowManager::shared().active.contains(&self.handle)
    }

    fn draw_frame(&self) {
        if let Some(bitmap) = &self.bitmap {
            let is_active = self.is_active();

            if self.style.contains(WindowStyle::BORDER) {
                let q = WINDOW_BORDER_SHADOW_PADDING;
                let rect = Rect::from(bitmap.size());
                for n in 0..q {
                    let rect = rect.insets_by(EdgeInsets::padding_all(n));
                    let light = 1 + n as u8;
                    let color = Color::TRANSPARENT.set_opacity(light * light);
                    bitmap.draw_rect(rect, color);
                }
                let shared = WindowManager::shared();
                let corner = &shared.resources.corner_shadow;
                bitmap.blt(
                    corner,
                    Point::new(0, 0),
                    Rect::new(0, 0, q, q),
                    BltOption::empty(),
                );
                bitmap.blt(
                    corner,
                    Point::new(rect.width() - q, 0),
                    Rect::new(q, 0, q, q),
                    BltOption::empty(),
                );
                bitmap.blt(
                    corner,
                    Point::new(0, rect.height() - q),
                    Rect::new(0, q, q, q),
                    BltOption::empty(),
                );
                bitmap.blt(
                    corner,
                    Point::new(rect.width() - q, rect.height() - q),
                    Rect::new(q, q, q, q),
                    BltOption::empty(),
                );
            }
            if self.style.contains(WindowStyle::TITLE) {
                let shared = WindowManager::shared();
                let pad_x = 8;
                let pad_left = WINDOW_BORDER_SHADOW_PADDING + pad_x;

                let rect = self.title_frame();
                bitmap.fill_rect(
                    rect,
                    if is_active {
                        WINDOW_ACTIVE_TITLE_BG_COLOR
                    } else {
                        WINDOW_INACTIVE_TITLE_BG_COLOR
                    },
                );

                if false {
                    let close = &shared.resources.close_button;
                    let close_pad_v = (rect.height() - close.height()) / 2;
                    let close_pad_h = close_pad_v + 8;
                    bitmap.blt(
                        close,
                        Point::new(
                            rect.x() + rect.width() - close.width() - close_pad_h,
                            rect.y() + close_pad_v,
                        ),
                        close.bounds(),
                        BltOption::empty(),
                    );
                }
                let pad_right = rect.height();

                if let Some(text) = self.title() {
                    let font = FontDriver::system_font();
                    let mut rect = rect;
                    let pad_y = (rect.height() - font.height()) / 2;
                    rect.origin.y += pad_y;
                    rect.size.height -= pad_y * 2;
                    rect.origin.x += pad_left;
                    rect.size.width -= pad_left + pad_right;
                    let mut rect2 = rect;
                    rect2.origin += Point::new(1, 1);
                    let mut ats =
                        AttributedString::with(text, font, WINDOW_ACTIVE_TITLE_SHADOW_COLOR);
                    if is_active {
                        ats.set_color(WINDOW_ACTIVE_TITLE_SHADOW_COLOR);
                        ats.draw(&bitmap, rect2);
                        ats.set_color(WINDOW_ACTIVE_TITLE_FG_COLOR);
                        ats.draw(&bitmap, rect);
                    } else {
                        ats.set_color(WINDOW_INACTIVE_TITLE_FG_COLOR);
                        ats.draw(&bitmap, rect);
                    }
                }
            }
        }
    }

    fn invalidate(&mut self) {
        if let Some(bitmap) = self.bitmap.as_ref() {
            if let Some(view) = self.view.as_mut() {
                view.layout_if_needed();
                if let Some(ctx) =
                    bitmap.view(Rect::from(self.frame.size).insets_by(self.content_insets))
                {
                    view.draw_in_context(ctx);
                }
            }
        }
        self.invalidate_rect(self.actual_bounds());
    }

    fn invalidate_rect(&self, rect: Rect<isize>) {
        if self.attributes.contains(WindowAttributes::VISIBLE) {
            self.draw_to_screen(rect, true);
        }
    }

    fn set_title_array(array: &mut [u8; WINDOW_TITLE_LENGTH], title: &str) {
        let mut i = 1;
        for c in title.bytes() {
            if i >= WINDOW_TITLE_LENGTH {
                break;
            }
            array[i] = c;
            i += 1;
        }
        array[0] = i as u8;
    }

    fn set_title(&mut self, title: &str) {
        RawWindow::set_title_array(&mut self.title, title);
        self.draw_frame();
        self.invalidate_rect(self.title_frame());
    }

    fn title<'a>(&self) -> Option<&'a str> {
        let len = self.title[0] as usize;
        match len {
            0 => None,
            _ => core::str::from_utf8(unsafe { core::slice::from_raw_parts(&self.title[1], len) })
                .ok(),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct WindowLevel(pub u8);

impl WindowLevel {
    pub const ROOT: WindowLevel = WindowLevel(0);
    pub const DESKTOP_ITEMS: WindowLevel = WindowLevel(1);
    pub const NORMAL: WindowLevel = WindowLevel(32);
    pub const FLOATING: WindowLevel = WindowLevel(64);
    pub const POPUP_BARRIER: WindowLevel = WindowLevel(96);
    pub const POPUP: WindowLevel = WindowLevel(97);
    pub const POINTER: WindowLevel = WindowLevel(127);
}

pub struct WindowBuilder {
    pub frame: Rect<isize>,
    pub style: WindowStyle,
    pub level: WindowLevel,
    pub bg_color: Color,
    pub bitmap: Option<Box<Bitmap>>,
    pub title: [u8; WINDOW_TITLE_LENGTH],
    pub no_bitmap: bool,
}

impl WindowBuilder {
    pub fn new(title: &str) -> Self {
        let window = Self {
            frame: Rect::new(isize::MIN, isize::MIN, 300, 300),
            level: WindowLevel::NORMAL,
            style: WindowStyle::DEFAULT,
            bg_color: Color::WHITE,
            bitmap: None,
            title: [0; WINDOW_TITLE_LENGTH],
            no_bitmap: false,
        };
        window.title(title).style(WindowStyle::DEFAULT)
    }

    #[inline]
    pub fn build(mut self) -> WindowHandle {
        let screen_bounds =
            WindowManager::main_screen_bounds().insets_by(WindowManager::shared().screen_insets);
        let shadow_insets = if self.style.contains(WindowStyle::BORDER) {
            EdgeInsets::padding_all(WINDOW_BORDER_SHADOW_PADDING)
        } else {
            EdgeInsets::zero()
        };
        let window_insets = self.style.as_content_insets();
        let content_insets = window_insets + shadow_insets;
        let mut frame = self.frame;
        if self.style.contains(WindowStyle::NAKED) {
            frame.size += window_insets;
        }
        if frame.x() == isize::MIN {
            frame.origin.x = (screen_bounds.width() - frame.width()) / 2;
        } else if frame.x() < 0 {
            frame.origin.x += screen_bounds.x() + screen_bounds.width();
        }
        if frame.y() == isize::MIN {
            frame.origin.y = (screen_bounds.height() - frame.height()) / 2;
        } else if frame.y() < 0 {
            frame.origin.y += screen_bounds.y() + screen_bounds.height();
        }
        frame.origin -= Point::new(shadow_insets.left, shadow_insets.top);
        frame.size += shadow_insets;

        if self.style.contains(WindowStyle::FLOATING) {
            self.level = WindowLevel::FLOATING;
        }

        if !self.no_bitmap && self.bitmap.is_none() {
            let bitmap = Bitmap::new(frame.width() as usize, frame.height() as usize, true);
            bitmap.fill_rect(Rect::from(bitmap.size()), self.bg_color);
            self.bitmap = Some(Box::new(bitmap));
        }

        let attributes = if self.level == WindowLevel::ROOT {
            WindowAttributes::new(WindowAttributes::VISIBLE)
        } else {
            WindowAttributes::EMPTY
        };

        let handle = WindowManager::next_window_handle();
        let window = RawWindow {
            handle,
            next: None,
            frame,
            shadow_insets,
            content_insets,
            style: self.style,
            level: self.level,
            bg_color: self.bg_color,
            bitmap: self.bitmap,
            title: self.title,
            attributes,
            view: None,
        };
        // window.draw_frame();
        WindowManager::add(Box::new(window));
        if !self.style.contains(WindowStyle::NAKED) {
            handle.load_view_if_needed();
        }
        handle
    }

    #[inline]
    pub fn style(mut self, style: WindowStyle) -> Self {
        self.style = style;
        self
    }

    #[inline]
    pub fn style_add(mut self, style: WindowStyle) -> Self {
        self.style |= style;
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        RawWindow::set_title_array(&mut self.title, title);
        self
    }

    #[inline]
    const fn level(mut self, level: WindowLevel) -> Self {
        self.level = level;
        self
    }

    #[inline]
    pub const fn frame(mut self, frame: Rect<isize>) -> Self {
        self.frame = frame;
        self
    }

    #[inline]
    pub const fn center(mut self) -> Self {
        self.frame.origin = Point::new(isize::MIN, isize::MIN);
        self
    }

    #[inline]
    pub const fn origin(mut self, origin: Point<isize>) -> Self {
        self.frame.origin = origin;
        self
    }

    #[inline]
    pub const fn size(mut self, size: Size<isize>) -> Self {
        self.frame.size = size;
        self
    }

    #[inline]
    pub const fn bg_color(mut self, bg_color: Color) -> Self {
        self.bg_color = bg_color;
        self
    }

    #[inline]
    pub const fn no_bitmap(mut self) -> Self {
        self.no_bitmap = true;
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowHandle(NonZeroUsize);

impl WindowHandle {
    #[inline]
    fn new(val: usize) -> Option<Self> {
        NonZeroUsize::new(val).map(|x| Self(x))
    }

    #[inline]
    #[track_caller]
    fn update<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut RawWindow) -> R,
    {
        let shared = WindowManager::shared();
        let window = shared.get_mut(self).unwrap();
        f(window)
    }

    #[inline]
    #[track_caller]
    fn as_ref<'a>(&self) -> &'a RawWindow {
        let shared = WindowManager::shared();
        shared.get(self).as_ref().unwrap()
    }

    // :-:-:-:-:

    pub fn set_title(self, title: &str) {
        self.update(|window| {
            window.set_title(title);
        });
    }

    pub fn title<'a>(self) -> Option<&'a str> {
        self.as_ref().title()
    }

    pub fn set_bg_color(self, color: Color) {
        self.update(|window| {
            window.bg_color = color;
        });
        if let Some(bitmap) = self.bitmap() {
            bitmap.fill_rect(bitmap.bounds(), color);
        }
        self.invalidate();
    }

    #[inline]
    pub(crate) fn bitmap(self) -> Option<&'static Box<Bitmap>> {
        self.as_ref().bitmap.as_ref()
    }

    #[inline]
    pub fn frame(self) -> Rect<isize> {
        self.as_ref().frame()
    }

    pub fn set_frame(self, rect: Rect<isize>) {
        self.update(|window| {
            window.set_frame(rect);
        });
    }

    #[inline]
    pub fn content_insets(self) -> EdgeInsets<isize> {
        self.as_ref().content_insets
    }

    #[inline]
    pub fn move_by(self, delta: Point<isize>) {
        let mut new_rect = self.frame();
        new_rect.origin += delta;
        self.set_frame(new_rect);
    }

    #[inline]
    pub fn move_to(self, new_origin: Point<isize>) {
        let mut new_rect = self.frame();
        new_rect.origin = new_origin;
        self.set_frame(new_rect);
    }

    #[inline]
    pub fn resize_to(self, new_size: Size<isize>) {
        let mut new_rect = self.frame();
        new_rect.size = new_size;
        self.set_frame(new_rect);
    }

    pub fn show(self) {
        WindowManager::synchronized(|| unsafe {
            WindowManager::add_hierarchy(self);
        });
        self.as_ref().draw_frame();
        self.invalidate();
    }

    pub fn hide(self) {
        let shared = WindowManager::shared();
        let frame = self.as_ref().frame;
        if shared.active.contains(&self) {
            shared.active = None;
        }
        if shared.captured.contains(&self) {
            shared.captured = None;
        }
        WindowManager::synchronized(|| unsafe {
            WindowManager::remove_hierarchy(self);
        });
        WindowManager::invalidate_screen(frame);
    }

    #[inline]
    pub fn set_active(self) {
        WindowManager::set_active(Some(self));
    }

    #[inline]
    pub fn invalidate_rect(self, rect: Rect<isize>) {
        self.as_ref().invalidate_rect(rect);
    }

    #[inline]
    pub fn invalidate(self) {
        self.update(|window| window.invalidate());
    }

    pub fn draw<F>(self, f: F) -> Result<(), WindowDrawingError>
    where
        F: FnOnce(&Bitmap) -> (),
    {
        self.draw_in_rect(self.frame().size().into(), f)
    }

    pub fn draw_in_rect<F>(self, rect: Rect<isize>, f: F) -> Result<(), WindowDrawingError>
    where
        F: FnOnce(&Bitmap) -> (),
    {
        let window = self.as_ref();
        let bitmap = match window.bitmap.as_ref() {
            Some(bitmap) => bitmap,
            None => return Err(WindowDrawingError::NoBitmap),
        };
        let bounds = Rect::from(window.frame.size).insets_by(window.content_insets);
        let coords = match Coordinates::from_rect(Rect::new(
            rect.x() + bounds.x(),
            rect.y() + bounds.y(),
            isize::min(rect.width(), bounds.width()),
            isize::min(rect.height(), bounds.height()),
        )) {
            Some(coords) => coords,
            None => return Err(WindowDrawingError::InconsistentCoordinates),
        };
        if coords.left > coords.right || coords.top > coords.bottom {
            return Err(WindowDrawingError::InconsistentCoordinates);
        }

        let rect = coords.into();
        if let Some(bitmap) = bitmap.view(rect) {
            f(&bitmap);
            window.invalidate_rect(rect);
        } else {
            return Err(WindowDrawingError::InconsistentCoordinates);
        }
        Ok(())
    }

    #[inline]
    pub fn view<'a>(self) -> Option<&'a mut Box<dyn ViewTrait>> {
        let shared = WindowManager::shared();
        let window = shared.get_mut(&self).unwrap();
        window.view.as_mut()
    }

    pub fn load_view_if_needed(self) {
        if self.as_ref().view.is_none() {
            let mut view = View::with_frame(self.frame().size().into());
            self.update(|window| {
                view.set_background_color(window.bg_color);
                window.view = Some(view);
                if let Some(view) = window.view.as_mut() {
                    view.move_to(Some(self));
                }
            });
        }
    }
}

#[derive(Debug)]
pub enum WindowDrawingError {
    NoBitmap,
    InconsistentCoordinates,
}