// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//DocumentNode level transformations
pub mod util;

//simple paragraph format <P>
pub mod paragraph;

//Basic text format including font changes like bold, underline, italic
pub mod format_const;
pub mod t_attributes;
pub mod t_formats;
pub mod text_formatter;

use crate::paragraph::Pblock;
use crate::text_formatter::TextFormat;
use node_tree::format_trait::FormatTait;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static P_FORMAT: Lazy<Arc<dyn FormatTait + Send + Sync>> =
    Lazy::new(|| Arc::new(Pblock::new()));
pub static TEXT_FORMAT: Lazy<Arc<dyn FormatTait + Send + Sync>> =
    Lazy::new(|| Arc::new(TextFormat::new()));
