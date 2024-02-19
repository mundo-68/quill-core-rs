// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use wasm_bindgen::prelude::*;
use web_sys::{Document, Window};

thread_local! {
    pub static WINDOW: Window = web_sys::window().unwrap_throw();
    pub static DOCUMENT: Document = WINDOW.with(|w| w.document().unwrap_throw());
    //pub static BODY: HtmlElement= DOCUMENT.with(|w| w.body().unwrap_throw());
    //pub static BODY:HtmlElement = DOCUMENT.body().expect("Could not find body");
    //pub static HISTORY: History = WINDOW.with(|w| w.history().unwrap_throw());
}
