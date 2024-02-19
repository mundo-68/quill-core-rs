// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Doc node link error: We have a non empty block, but can not find a previous.")]
    NonEmptyBlockCanNotTraversePrev,
    #[error("Backspace cursor beyond start of document.")]
    BackspaceBeyondStart,
    #[error("Advance cursor beyond end of document.")]
    AdvanceBeyondEnd,
    #[error("Unexpected cursor position with value: None.")]
    UnexepectedCursorPosNone,
}
