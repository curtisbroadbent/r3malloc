#![no_std] // Disables Rust heap
#![allow(dead_code)] // FIXME: have it here so there's no warning spam

mod defines;
mod heap;
mod log;
mod pagemap;
mod pages;
mod r3malloc;
mod size_classes;
mod tcache;

use heap::Anchor;
use libc_print::libc_println;
use likely_stable::{likely, unlikely};
use core::ptr::{null_mut, copy};
use core::slice;
use pagemap::SPAGEMAP;

extern crate libc;

// FIXME: Dummy code as a POC (see tests/dummy.c)
#[no_mangle]
pub extern "C" fn test() -> u32 {
    let mut anch: Anchor = Anchor::new();

    anch.set_avail(128);
    libc_println!("Hello from Rust: {}", anch.get_avail());
    let _dummy = heap::Descriptor::alloc();
    let _dummy2 = heap::Descriptor::alloc();

    use heap::Descriptor;
    use r3malloc::{heap_pop_partial, heap_push_partial, init_malloc, HEAPS};
    init_malloc();

    // test heap push/pop
    let heap = unsafe { &mut HEAPS[1] };
    let mut stuff: *mut Descriptor = heap_pop_partial(heap);
    let list = heap.get_partial_list();
    libc_println!("\ninitial list of heap: {:?}\n", list);
    _dummy.set_heap(heap);
    _dummy.set_block_size(10);
    _dummy.set_maxcount(20);
    libc_println!("desc to add: {:?}\n", _dummy);
    libc_println!("1st pop is null: {}\n", stuff.is_null());
    heap_push_partial(_dummy);
    stuff = heap_pop_partial(heap);
    libc_println!("2nd pop is null: {}", stuff.is_null());
    libc_println!("2nd pop object: {:?}\n", unsafe { &mut *stuff });
    stuff = heap_pop_partial(heap);
    libc_println!("3rd pop is null: {}\n", stuff.is_null());

    anch.get_avail()
}

#[no_mangle]
pub extern "C" fn malloc(size: usize) -> *mut libc::c_void {
    r3malloc::do_malloc(size) as *mut libc::c_void
}

#[no_mangle]
pub extern "C" fn free(ptr: *mut libc::c_void) {
    r3malloc::do_free(ptr as *mut u8)
}

#[no_mangle]
pub extern "C" fn calloc(n: usize, size: usize) -> *mut libc::c_void {
    let alloc_size = n * size;

    // overflow check
    // @todo: expensive, need to optimize
    if unlikely(n == 0 || alloc_size / n != size) {
        return null_mut();
    }

    let ptr = r3malloc::do_malloc(alloc_size);

    // calloc returns zero-filled memory
    // @todo: optimize, memory may be already zero-filled
    //  if coming directly from OS
    if likely(ptr != null_mut()) {
        unsafe { slice::from_raw_parts_mut(ptr, alloc_size).fill(0x0); }
    }

    ptr as *mut libc::c_void
}

#[no_mangle]
pub extern "C" fn realloc(ptr: *mut libc::c_void, size: usize) -> *mut libc::c_void {
    let mut block_size = 0;

    if likely(!ptr.is_null()) {
        let info = unsafe { SPAGEMAP.get_page_info(ptr as *mut u8) };
        let desc = info.get_desc();
        assert!(!desc.is_null());

        block_size = unsafe { (& *desc).get_block_size() };

        if unlikely(size == 0) {
            r3malloc::do_free(ptr as *mut u8);
            return null_mut();
        }

        if unlikely(size <= block_size as usize) {
            return ptr;
        }
    }

    let new_ptr = r3malloc::do_malloc(size) as *mut libc::c_void;
    if likely(!ptr.is_null() && !new_ptr.is_null()) {
        unsafe { copy(ptr, new_ptr, block_size as usize) };
        r3malloc::do_free(ptr as *mut u8);
    }

    return new_ptr;
}

use core::panic::PanicInfo;

// Called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    libc_println!("{}", info);

    loop {}
}
