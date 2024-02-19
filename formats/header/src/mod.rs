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
use core_formats::util::lookup::AttributesLookup;
use core_formats::util::node_morph::delete_node;
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use delta::types::attr_val::AttrVal::Null;
use dom::dom_element::DomElement;
use log::error;
use node_tree::cursor::Cursor;
use node_tree::doc_node::DocumentNode;
use node_tree::format_trait::FormatTait;
use once_cell::sync::OnceCell;
use std::sync::Arc;

pub static NAME_HEADER: &'static str = "heading"; //registry label

static HX_TAG: &'static str = "H"; //HTML tag
static HEADER_ATTR_KEY: &'static str = "heading"; //attribute key

//FIXME: Default structure, but is it used in this scope?
static ATTRIBUTES: OnceCell<AttributesLookup> = OnceCell::new();
pub(crate) fn initialise() {
    if let Some(_attr) = ATTRIBUTES.get() {
        return;
    }
    let mut attr = AttributesLookup::new(1);
    attr.fill_one(HEADER_ATTR_KEY, HX_TAG);
    ATTRIBUTES
        .set(attr)
        .expect("did you call header::initialise() twice?");
}

/// # HeaderBlock
/// 
/// This is a line format, so we replace the previous paragraph, shown here as a string without formatting<br>
///  - `{ insert(header 1)}, {insert(\n), attributes:{heading:1}}` --> `<H1>header 1</H1>`
///  - `{ insert(header 2)}, {insert(\n), attributes:{heading:2}}` --> `<H2>header 2</H2>`
///  - `{ insert(header 3)}, {insert(\n), attributes:{heading:3}}` --> `<H3>header 3</H3>`
///
/// The attribute in the delta should show: header, and the value should show which one 1,2,3
pub struct HeaderBlock {}
impl HeaderBlock {
    pub fn new() -> Self {
        initialise();
        block_format::initialise();
        HeaderBlock {}
    }
}

impl FormatTait for HeaderBlock {
    fn create(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let val = operation.get_attributes().get(HEADER_ATTR_KEY).unwrap();
        let idx = val.number_val().unwrap();
        let name = format!("H{:?}", idx);
        let element = DomElement::new(&name);
        block_format::apply(&element, operation.get_attributes())?;
        let doc_node = DocumentNode::new_element(element, formatter);
        doc_node.set_operation(operation);
        return Ok(Arc::new(doc_node));
    }

    fn format_name(&self) -> &'static str {
        NAME_HEADER
    }

    fn is_text_format(&self) -> bool {
        false
    }

    fn block_remove_attr(&self) -> Attributes {
        let mut attr = Attributes::default();
        attr.insert(HEADER_ATTR_KEY, Null);
        attr
    }

    fn applies(&self, delta: &DeltaOperation) -> Result<bool> {
        if delta.insert_value().is_string() && delta.insert_value().str_val()? == "\n" {
            if delta.get_attributes().contains_key(HEADER_ATTR_KEY) {
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
        panic!("HeaderFormat::split_leaf() - Error. ");
    }

    //We do not merge so we always flag it to be not the same
    fn is_same_format<'a>(&self, _left: &Arc<DocumentNode>, _right: &Arc<DocumentNode>) -> bool {
        return false;
    }

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
        block_node.get_operation().remove_attribute(HEADER_ATTR_KEY);
        error!(
            "HEADING un_block_transform() - attr removed = {:?}",
            block_node.get_operation().get_attributes()
        );
        un_block_transform(block_node, cursor)
    }

    fn delete_leaf_segment(
        &self,
        _doc_node: &Arc<DocumentNode>,
        _at: usize,
        _length: usize,
    ) -> Result<()> {
        panic!("HeaderFormat::delete() - Error. Block has length 1, so use the other delete function...");
    }

    fn delete_node(&self, doc_node: &Arc<DocumentNode>) {
        assert!(doc_node.get_doc_dom_node().get_node_name().contains(HX_TAG));
        delete_node(doc_node);
    }

    fn isolate(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        Ok(doc_node.clone())
    }

    //We do not merge headers
    //If the author wants 2x the same header consecutively than that is what she wants right?
    fn try_merge(&self, _cursor: &Cursor, _block_node: &Arc<DocumentNode>) -> Result<()> {
        Ok(())
    }
}
