#![no_std]
use linked_list_allocator::Heap;
use rps2_thread::mutex::{Mutex, MutexGuard};

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, NonNull};

struct Allocator(Option<Mutex<Heap>>);

impl Allocator {
    fn heap(&self) -> MutexGuard<'_, Heap> {
        self.0
            .as_ref()
            .expect("Allocator not yet initialized!")
            .lock()
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.heap()
            .allocate_first_fit(layout)
            .ok()
            .map_or(ptr::null_mut(), NonNull::as_ptr)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.heap().deallocate(NonNull::new_unchecked(ptr), layout);
    }
}

#[global_allocator]
static mut ALLOCATOR: Allocator = Allocator(None);

pub unsafe fn init(start: *mut u8, end: *mut u8) {
    ALLOCATOR.0 = Some(Mutex::new(Heap::new(start, end as usize - start as usize)));
}
