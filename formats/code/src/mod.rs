// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use anyhow::Result;
use core_formats::util::block::{
    apply_attributes, block_transform, drop_attributes, un_block_transform,
};
use core_formats::util::block_format;
use core_formats::util::node_morph::delete_node;
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use delta::types::attr_val::AttrVal;
use delta::types::attr_val::AttrVal::Null;
use dom::dom_element::DomElement;
use node_tree::cursor::Cursor;
use node_tree::doc_node::DocumentNode;
use node_tree::format_trait::FormatTait;
use std::sync::Arc;

pub static NAME_CODE: &'static str = "CODE"; //registry label

static CODE_TAG: &'static str = "SPAN";
static CODE_ATTR_KEY: &'static str = "code-block";
static CODE_CLASS: &'static str = "ql-pre";

/// # CodeBlock
///
/// ``` bash
///   {insert: "hello "}
///   {attributes: {code-block: true}, insert: "↵"}
///   {insert: "sweet"}
///   {attributes: {code-block: true}, insert: "↵"}
///   {insert: "world"}
///   {attributes: {code-block: true}, insert: "↵"}
/// ```
///
/// Gives as HTML:
/// ```bash
///     <span class="ql-pre" ">hello \n</span>
///     <span class="ql-pre" ">sweet\n</span>
///     <span class="ql-pre" ">world\n</span>
/// ```
///
/// We then need CSS formatting
/// ```bash
///     span.ql-pre {
///        white-space: pre;
///        font-family: monospace;
///        display: block;
///     }
/// ```
/// Note: using the pre-tag is also possible, but then we get multiple `<text>` tags in one `<pre>` tag.
/// that is not as the HTML dom would want it. Normally all the text nodes are merged in to one,
/// but that would mean that we have no 1:1 relation anymore between `<text>` nodes, doc_nodes, and
/// delta operations
pub struct CodeBlock {}
impl CodeBlock {
    pub fn new() -> Self {
        block_format::initialise();
        CodeBlock {}
    }
}

impl FormatTait for CodeBlock {
    fn create(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let element = DomElement::new(CODE_TAG);
        element.set_class(CODE_CLASS);
        let doc_node = DocumentNode::new_element(element, formatter);
        doc_node.set_operation(operation);
        return Ok(Arc::new(doc_node));
    }

    fn format_name(&self) -> &'static str {
        NAME_CODE
    }

    fn is_text_format(&self) -> bool {
        false
    }

    fn block_remove_attr(&self) -> Attributes {
        let mut attr = Attributes::default();
        attr.insert(CODE_ATTR_KEY, Null);
        attr
    }

    fn applies(&self, delta: &DeltaOperation) -> Result<bool> {
        if delta.insert_value().is_string() && delta.insert_value().str_val()? == "\n" {
            if delta.get_attributes().contains_key(CODE_ATTR_KEY) {
                return Ok(true);
            }
        }
        return Ok(false);
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

    fn split_leaf(&self, _cursor: &Cursor) -> Result<()> {
        panic!("CodeFormat::split_leaf() - Error. ");
    }

    fn is_same_format(&self, left: &Arc<DocumentNode>, right: &Arc<DocumentNode>) -> bool {
        let left_a = left.get_operation().get_attributes().clone();
        let right_a = right.get_operation().get_attributes().clone();
        if left_a.contains_key(CODE_ATTR_KEY) && right_a.contains_key(CODE_ATTR_KEY) {
            if let Some(AttrVal::Bool(l)) = left_a.get(CODE_ATTR_KEY) {
                if let Some(AttrVal::Bool(r)) = right_a.get(CODE_ATTR_KEY) {
                    return l == r;
                }
            }
        }
        return false;
    }

    //Only called for block-formats; the block is inserted in the parent.
    fn block_transform(
        &self,
        cursor: &Cursor,
        block_node: &Arc<DocumentNode>,
        delta: DeltaOperation,
        format: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        block_transform(block_node, delta, format, cursor)
    }

    fn un_block_transform(
        &self,
        cursor: &Cursor,
        block_node: &Arc<DocumentNode>,
    ) -> Result<Arc<DocumentNode>> {
        block_node.get_operation().remove_attribute(CODE_ATTR_KEY);
        un_block_transform(block_node, cursor)
    }

    fn delete_leaf_segment(
        &self,
        _doc_node: &Arc<DocumentNode>,
        _at: usize,
        _length: usize,
    ) -> Result<()> {
        panic!(
            "CodeFormat::delete() - Error. Block has length 1, so use the other delete function..."
        );
    }

    fn delete_node(&self, doc_node: &Arc<DocumentNode>) {
        assert!(doc_node
            .get_doc_dom_node()
            .get_node_name()
            .contains(CODE_TAG));
        delete_node(doc_node);
    }

    fn isolate(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        return Ok(doc_node.clone());
    }

    fn try_merge(&self, _cursor: &Cursor, _block_node: &Arc<DocumentNode>) -> Result<()> {
        Ok(())
    }
}
