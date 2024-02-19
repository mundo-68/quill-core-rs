// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use anyhow::Result;
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use delta::types::attr_val::AttrVal::Null;
use dom::dom_element::DomElement;
use node_tree::cursor::Cursor;
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::unlink;
use node_tree::format_trait::FormatTait;
use std::sync::Arc;

pub static NAME_SOFT_BREAK: &str = "SOFT_BREAK";

///HTML tag
static SOFT_BREAK_TAG: &str = "BR";

///Insert::map keys
const BREAK: &str = "page_break";

/// # SoftBreak
///
/// Tag which belongs to the HTML element `<BR/>` must be empty !! <br>
/// So we treat this format as a line format not a BLOCK format.
///
/// We have 2 soft break incarnations:
///  - Normal soft break which lives in a DeltaOperation document
///  - Automatically inserted (temporary) place holders to display an empty block node
///
/// The automatically inserted place holders shall have empty delta, and are NOT recognized
/// when inserted as a normal delta operation:
/// ```bash
///  Insert{
///    ""
/// }
///
/// The normal page break shows as a character of length 1
/// when inserted as a normal delta operation:
/// ```bash
///  Insert{
///    {"page_break", "true"}
/// }
/// ```
#[derive(Default)]
pub struct SoftBreak {}

impl SoftBreak {
    /// Creates a format with default interface
    pub fn new() -> Self {
        SoftBreak {}
    }
}

impl FormatTait for SoftBreak {
    fn create(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let br = DomElement::new(SOFT_BREAK_TAG);
        let dn = Arc::new(DocumentNode::new_element(br, formatter.clone()));
        dn.set_operation(operation);
        Ok(dn)
    }

    fn format_name(&self) -> &'static str {
        NAME_SOFT_BREAK
    }

    fn is_text_format(&self) -> bool {
        true
    }

    fn block_remove_attr(&self) -> Attributes {
        let mut attr = Attributes::default();
        attr.insert(BREAK, Null);
        attr
    }

    fn applies(&self, op: &DeltaOperation) -> Result<bool> {
        if op.insert_value().is_map() {
            let val = op.insert_value().map_val().unwrap();
            if val.contains_key(BREAK) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn apply_line_attributes(
        &self,
        doc_node: &Arc<DocumentNode>,
        _attr: &Attributes,
        _formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        Ok(doc_node.clone())
    }

    fn drop_line_attributes(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        Ok(doc_node.clone())
    }

    /// This does not apply to any block format.
    fn split_leaf(&self, _cursor: &Cursor) -> Result<()> {
        panic!("{} -- You called: {:?}()", NAME_SOFT_BREAK, "split_leaf");
    }

    fn is_same_format(&self, _left: &Arc<DocumentNode>, right: &Arc<DocumentNode>) -> bool {
        if NAME_SOFT_BREAK.eq(right.get_formatter().format_name()) {
            return true;
        }
        false
    }

    fn block_transform(
        &self,
        _cursor: &Cursor,
        _block_node: &Arc<DocumentNode>,
        _delta: DeltaOperation,
        _format: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        panic!("{} -- You called: {}()", NAME_SOFT_BREAK, "block_transform");
    }

    fn un_block_transform(
        &self,
        _cursor: &Cursor,
        _block_node: &Arc<DocumentNode>,
    ) -> Result<Arc<DocumentNode>> {
        panic!(
            "{} -- You called: {}()",
            NAME_SOFT_BREAK, "un_block_transform"
        );
    }

    fn delete_leaf_segment(
        &self,
        _doc_node: &Arc<DocumentNode>,
        _at: usize,
        _length: usize,
    ) -> Result<()> {
        panic!("{} -- You called: {}()", NAME_SOFT_BREAK, "delete");
    }

    fn delete_node(&self, doc_node: &Arc<DocumentNode>) {
        assert_eq!(doc_node.get_doc_dom_node().get_node_name(), SOFT_BREAK_TAG);
        let parent = doc_node.get_parent().unwrap();
        unlink(&parent, doc_node);
    }

    fn isolate(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        Ok(doc_node.clone())
    }

    /// P-blocks are not merged even when the are in consecutive order as siblings
    fn try_merge(&self, _cursor: &Cursor, _block_node: &Arc<DocumentNode>) -> Result<()> {
        Ok(())
    }
}
