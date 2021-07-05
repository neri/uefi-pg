//! First In First Out

use crate::arch::cpu::Cpu;
use crate::sync::spinlock::SpinLoopWait;
use alloc::boxed::Box;
use core::mem::{self, MaybeUninit};
use core::{cell::UnsafeCell, sync::atomic::*};

/// First In First Out
pub struct ConcurrentFifo<T> {
    head: AtomicUsize,
    tail: AtomicUsize,
    mask: usize,
    one_lap: usize,
    data: *mut Slot<T>,
}

unsafe impl<T: Send> Send for ConcurrentFifo<T> {}

unsafe impl<T: Send> Sync for ConcurrentFifo<T> {}

impl<T: Sized> ConcurrentFifo<T> {
    #[inline]
    pub fn with_capacity(size: usize) -> Self {
        let capacity = (size + 1).next_power_of_two();
        let mask = capacity - 1;

        let data = {
            let mut boxed: Box<[Slot<T>]> = (0..capacity).map(|v| Slot::new(v)).collect();
            let ptr = boxed.as_mut_ptr();
            mem::forget(boxed);
            ptr
        };

        Self {
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            mask,
            one_lap: capacity,
            data,
        }
    }

    #[inline]
    pub fn enqueue(&self, value: T) -> Result<(), T> {
        unsafe { Cpu::without_interrupts(|| self._enqueue(value)) }
    }

    #[inline]
    pub fn dequeue(&self) -> Option<T> {
        unsafe { Cpu::without_interrupts(|| self._dequeue()) }
    }

    #[track_caller]
    fn _enqueue(&self, data: T) -> Result<(), T> {
        assert!(
            unsafe { Cpu::is_interrupt_disabled() },
            "ConcurrentQueue.enqueue: Interrupts must be disabled."
        );

        let mut spin = SpinLoopWait::new();
        let mut tail = self.tail.load(Ordering::Relaxed);
        loop {
            if (tail + 1) & self.mask == self.head.load(Ordering::Relaxed) & self.mask {
                return Err(data);
            }
            let index = tail & self.mask;
            let slot = unsafe { &mut *self.data.add(index) };
            if slot.stamp() == tail {
                match self.tail.compare_exchange(
                    tail,
                    tail + 1,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(tail) => {
                        slot.write(data, tail + 1);
                        return Ok(());
                    }
                    Err(v) => tail = v,
                }
            }
            spin.wait();
        }
    }

    fn _dequeue(&self) -> Option<T> {
        let mut spin = SpinLoopWait::new();
        let mut head = self.head.load(Ordering::Relaxed);
        loop {
            let index = head & self.mask;
            if index == self.tail.load(Ordering::Relaxed) & self.mask {
                return None;
            }
            let slot = unsafe { &*self.data.add(index) };
            if slot.stamp() == head + 1 {
                fence(Ordering::SeqCst);
                match self.head.compare_exchange(
                    head,
                    head + 1,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        let data = slot.read();
                        slot.write_stamp(head + self.one_lap);
                        return Some(data);
                    }
                    Err(v) => head = v,
                }
            }
            spin.wait();
        }
    }
}

impl<T> Drop for ConcurrentFifo<T> {
    fn drop(&mut self) {
        // TODO
    }
}

struct Slot<T> {
    stamp: AtomicUsize,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> Slot<T> {
    #[inline]
    fn new(stamp: usize) -> Self {
        Self {
            stamp: AtomicUsize::new(stamp),
            value: UnsafeCell::new(MaybeUninit::zeroed()),
        }
    }

    #[inline]
    fn stamp(&self) -> usize {
        self.stamp.load(Ordering::Acquire)
    }

    #[inline]
    fn write_stamp(&self, val: usize) {
        self.stamp.store(val, Ordering::Release);
    }

    #[inline]
    fn write(&mut self, val: T, stamp: usize) {
        self.value.get_mut().write(val);
        fence(Ordering::SeqCst);
        self.write_stamp(stamp);
    }

    #[inline]
    fn read(&self) -> T {
        unsafe { mem::transmute_copy(&*self.value.get()) }
    }
}
