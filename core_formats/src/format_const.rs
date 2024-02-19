// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/// # format constants
///
/// These names are used in the Delta documents to define formats for text.
/// These can have either value `true`, or be absent from the attributes list.
/// Inverted delta operations may have these values also as `false` in order
/// to switch the format off.

pub const FORMAT_BOLD: &str = "bold";
pub const FORMAT_ITALIC: &str = "italic";
pub const FORMAT_UNDERLINE: &str = "underline";
pub const FORMAT_STRIKE: &str = "strike";
pub const FORMAT_SUB: &str = "subscript";
pub const FORMAT_SUP: &str = "superscript";
pub const FORMAT_DELETED: &str = "deleted";
pub const FORMAT_INSERTED: &str = "inserted";
pub const FORMAT_MARKED: &str = "marked";
pub const FORMAT_SMALL: &str = "small";

/// These names are used in the Delta documents to define attributes for text.
pub const TEXT_ATTR_FONT: &str = "font";
pub const TEXT_ATTR_SIZE: &str = "size";
pub const TEXT_ATTR_COLOR: &str = "color";
pub const TEXT_ATTR_BACK_GROUND: &str = "background";

/// At a minimum there shall be support for a paragraph format, and one text format.
/// These shall have default labels:
pub static NAME_P_BLOCK: &str = "F_P-BLOCK";
pub static NAME_TEXT: &str = "F_TEXT";
