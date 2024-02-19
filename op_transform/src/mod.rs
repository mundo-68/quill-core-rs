// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/// # op_transform
///
/// The operational transform has just a few methods:
///
/// - retain
/// - delete
/// - insert
///
/// These are implemented generically here. But for the different formats we need a different
/// detailed implementation. Hence we provide a format trait to "inject" that behaviour.
/// The operation itself is used to select the proper formatter
pub mod op_delete;
pub mod op_insert;
pub mod op_retain;

// The basic document starts with 1 root node
// The registry describes all formats supported.
// This registry is a static global variable.
// Any locally generated error returned is described  in the module error.
pub mod doc_root;
pub mod error;
pub mod registry;

pub mod auto_soft_break;

use cfg_if::cfg_if;
extern crate web_sys;
use log::Level;

// When the `console_error_panic_hook` feature is enabled, we can call the
// `set_panic_hook` function at least once during initialization, and then
// we will get better error messages if our code ever panics.
//
// For more details see
// https://github.com/rustwasm/console_error_panic_hook#readme
cfg_if! {
    if #[cfg(feature = "console_error_panic_hook")] {
        //use log::Level;
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

// When the `console_log` feature is enabled, forward log calls to the
// JS console.
cfg_if! {
    if #[cfg(feature = "console_log")] {
        pub fn init_log(level: Level) {
            // Best effort, ignore error if initialization fails.
            // (This could be the case if the logger is initialized multiple
            // times.)
            let _ = console_log::init_with_level(level);
        }
    } else {
        pub fn init_log(_level: Level) {}
    }
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[allow(unused_macros)]
macro_rules! log {
        ( $( $t:tt )* ) => {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}
