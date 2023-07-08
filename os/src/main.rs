#![no_std] // tell rustc not use the standard library
#![no_main] // the simplest way to disable the 'start' program to initialize env
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(strict_provenance)]
// customized tests
#![reexport_test_harness_main = "test_main"] // help us create new `main` entry for test
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![feature(pointer_byte_offsets)]

#[path = "boards/qemu.rs"]
mod board;

extern crate alloc;
#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod console;
mod config;
mod lang_items;
mod loader;
mod mm;
mod logging;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
pub mod trap;
mod timer;

use log::*;

// ch2-problems
mod stack_btrace;
use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

fn clear_bss() {
    extern "C" {
        static mut sbss: u64;
        static mut ebss: u64;
    }

    unsafe {
        (sbss as usize..ebss as usize).for_each(|ptr|{
                // use volatile to avoid compiler optimization
                (ptr as *mut u8).write_volatile(0);
            }
        );
    }
}

/// kernel log info
fn kernel_log_info() {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }
    logging::init();
    println!("[kernel] Hello, world!");
    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
}



#[no_mangle] // avoid compiler confusion
fn rust_main() {
    clear_bss();
    kernel_log_info();

    println!("[kernel] Hello, world!");
    mm::init();
    println!("[kernel] back to world!");
    // mm tests
    mm::heap_test();
    mm::frame_allocator_test();
    mm::remap_test();

    task::add_initproc();
    println!("after initproc!");

    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    loader::list_apps();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}

#[cfg(test)] // ensure this function only runs in test scenario
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    // use crate::board::QEMUExit;
    // crate::board::QEMU_EXIT_HANDLE.exit_success(); // CI autotest success
}