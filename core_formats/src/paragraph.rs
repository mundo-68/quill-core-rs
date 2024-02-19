// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::format_const::NAME_P_BLOCK;
use crate::util::block::{apply_attributes, drop_attributes};
use crate::util::node_morph::delete_node;
use crate::util::{block, block_format};
use anyhow::Result;
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use dom::dom_element::DomElement;
use node_tree::cursor::Cursor;
use node_tree::doc_node::DocumentNode;
use node_tree::format_trait::FormatTait;
use std::sync::Arc;

/// # P_BLOCK_TAG
/// Tag which belongs to the HTML element `<P>`
static P_BLOCK_TAG: &str = "P";

/// # Pblock
///
/// Formatting of a basic paragraph.
///
/// This will apply to any `DeltaOperation` which contains a `\n`
/// character. Check the `Pblock::apply()` function implementation in this trait for details.
///
/// To get most of of this formatter, it should be the last formatter registered in the
/// `Registry`. If not, it will "grab" any delta it can get, and other formats later in the
/// registry will not be able to respond to anything that comes by ...
#[derive(Default)]
pub struct Pblock {}

impl Pblock {
    pub fn new() -> Self {
        block_format::initialise();
        Pblock {}
    }
}

impl FormatTait for Pblock {
    fn create(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let element = DomElement::new(P_BLOCK_TAG);
        block_format::apply(&element, operation.get_attributes())?;
        let dn = Arc::new(DocumentNode::new_element(element, formatter.clone()));
        dn.set_operation(operation);

        Ok(dn)
    }

    fn format_name(&self) -> &'static str {
        NAME_P_BLOCK
    }

    //fn is_block_format(&self, _delta: &DeltaOperation) -> bool { return true; }

    fn is_text_format(&self) -> bool {
        false
    }

    fn block_remove_attr(&self) -> Attributes {
        Attributes::default()
    }

    fn applies(&self, delta: &DeltaOperation) -> Result<bool> {
        if delta.insert_value().is_string() && delta.insert_value().str_val()?.eq("\n") {
            return Ok(true);
        }
        Ok(false)
    }

    fn apply_line_attributes(
        &self,
        doc_node: &Arc<DocumentNode>,
        attr: &Attributes,
        _formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        apply_attributes(doc_node, attr)
    }

    fn drop_line_attributes(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        drop_attributes(doc_node)
    }

    /// This does not apply to any block format.
    fn split_leaf(&self, _cursor: &Cursor) -> Result<()> {
        panic!("{} -- You called: {}()", NAME_P_BLOCK, "split_leaf");
    }

    fn is_same_format(&self, _left: &Arc<DocumentNode>, right: &Arc<DocumentNode>) -> bool {
        if NAME_P_BLOCK.eq(right.get_formatter().format_name()) {
            return true;
        }
        false
    }

    fn block_transform(
        &self,
        cursor: &Cursor,
        block_node: &Arc<DocumentNode>,
        delta: DeltaOperation,
        format: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        block::block_transform(block_node, delta, format, cursor)
    }

    fn un_block_transform(
        &self,
        cursor: &Cursor,
        block_node: &Arc<DocumentNode>,
    ) -> Result<Arc<DocumentNode>> {
        block::un_block_transform(block_node, cursor)
    }

    fn delete_leaf_segment(
        &self,
        _doc_node: &Arc<DocumentNode>,
        _at: usize,
        _length: usize,
    ) -> Result<()> {
        panic!("{} -- You called: {}()", NAME_P_BLOCK, "delete");
    }

    fn delete_node(&self, doc_node: &Arc<DocumentNode>) {
        assert_eq!(doc_node.get_doc_dom_node().get_node_name(), P_BLOCK_TAG);
        delete_node(doc_node);
    }

    fn isolate(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        Ok(doc_node.clone())
    }

    /// P-blocks are not merged even when the are in consecutive order as siblings
    fn try_merge(&self, _cursor: &Cursor, _block_node: &Arc<DocumentNode>) -> Result<()> {
        Ok(())
    }
}
