// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

pub mod cursor; //Points to a document node in the document tree
pub mod doc_node; //node structure building the document
pub mod dom_cursor; //links the DOM cursor to a document node cursor
pub mod tree_traverse; //implements navigation in the tree //for displaying empty block nodes, used in root_node & op_transform module only

//Changing the structure
pub mod dom_doc_node;
pub mod dom_doc_tree_morph; //changes parent child relations taking an Arc<DocumentNode> as input (instead of "self") //links the document node to a HTML DOM element

//Rendering is done by separately implemented formats. All these renderers implement this trait
pub mod error;
pub mod format_trait;

pub static EDITOR_CLASS: &str = "ql-editor";
