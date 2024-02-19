// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use anyhow::Result;
use core_formats::util::lookup::{AttributesLookup, Attributor};
use core_formats::util::node_morph::delete_node;
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use dom::dom_element::DomElement;
use node_tree::cursor::Cursor;
use node_tree::doc_node::DocumentNode;
use node_tree::format_trait::FormatTait;
use once_cell::sync::OnceCell;
use std::sync::Arc;

pub static NAME_IMAGE: &'static str = "image"; //registry label

static IMAGE_TAG: &'static str = "img"; //html tag

static ATTRIBUTES: OnceCell<AttributesLookup> = OnceCell::new();
pub(crate) fn initialise() {
    if let Some(_attr) = ATTRIBUTES.get() {
        return;
    }
    let mut attr = AttributesLookup::new(4);
    attr.fill_one("alt", "alt");
    attr.fill_one("alt", "alt");
    attr.fill_one("height", "height");
    attr.fill_one("width", "width");
    ATTRIBUTES
        .set(attr)
        .expect("failed to set config. did you call read_config() twice?");
}

/// # ImageFormat
///
/// Insert an embedded object:
/// ```bash
/// {
///   insert: { image: 'octodex.github.com/images/labtocat.png' },
///   attributes: { alt: "Lab Octocat", width:500, height:600 }
/// }
/// ```
///
/// Results in:
/// ```bash
/// <img src="octodex.github.com/images/labtocat.png" alt="Lab Octocat" width="500" height="600">
/// ```
pub struct ImageFormat {}

impl ImageFormat {
    pub fn new() -> Self {
        initialise();
        ImageFormat {}
    }
}

//These are the transformations we can do to a document ...
//Implementation depends on type of DocumentNode, hence is not implemented here
impl FormatTait for ImageFormat {
    fn create(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let dom_el = DomElement::new(IMAGE_TAG);
        dom_el.set_attribute(
            IMAGE_TAG,
            &operation
                .insert_value()
                .map_val()
                .unwrap()
                .get(NAME_IMAGE)
                .unwrap()
                .str_val()?,
        );

        for (k, v) in Attributor::selected(operation.get_attributes(), ATTRIBUTES.get().unwrap()) {
            dom_el.set_attribute(k, &v.str_val()?)
        }

        let doc_node = DocumentNode::new_element(dom_el, formatter);
        doc_node.set_operation(operation);
        return Ok(Arc::new(doc_node));
    }

    fn format_name(&self) -> &'static str {
        NAME_IMAGE
    }

    fn is_text_format(&self) -> bool {
        true
    }

    fn block_remove_attr(&self) -> Attributes {
        panic!("Hey you called ImageFormat::block_remove_attr() on the TextFormatter format-trait implementation.");
    }

    fn applies(&self, delta: &DeltaOperation) -> Result<bool> {
        let val = delta.insert_value();
        if val.is_map() {
            let av = val.map_val()?.get(NAME_IMAGE).unwrap();
            if av.is_string() {
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
        let dom_el = doc_node.get_dom_element().unwrap();
        for (k, v) in Attributor::selected(attr, ATTRIBUTES.get().unwrap()) {
            dom_el.set_attribute(k, &v.str_val()?)
        }
        Ok(doc_node.clone())
    }

    fn drop_line_attributes(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        let dom_el = doc_node.get_dom_element().unwrap();
        let lookup = ATTRIBUTES.get().unwrap();
        for key in Attributor::all_key(lookup) {
            dom_el.remove_attribute(&key);
        }
        Ok(doc_node.clone())
    }

    fn split_leaf(&self, _cursor: &Cursor) -> Result<()> {
        //we have length 1 (always) so it can not be split
        panic!("ImageFormat::split_leaf() - Error.");
    }

    fn is_same_format(&self, _left: &Arc<DocumentNode>, _right: &Arc<DocumentNode>) -> bool {
        false //never merge 2 images :-)
    }

    fn block_transform(
        &self,
        _cursor: &Cursor,
        _block_node: &Arc<DocumentNode>,
        _delta: DeltaOperation,
        _new_transducer: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        panic!("ImageFormat::block_transform() - Error.");
    }

    fn un_block_transform(
        &self,
        _cursor: &Cursor,
        _block_node: &Arc<DocumentNode>,
    ) -> Result<Arc<DocumentNode>> {
        panic!("ImageFormat::un_block_transform() - Error.");
    }

    //length shall be equal or less than own length
    fn delete_leaf_segment(
        &self,
        _doc_node: &Arc<DocumentNode>,
        _at: usize,
        _length: usize,
    ) -> Result<()> {
        panic!("ImageFormat::Delete() - Image has length 1, so use the other delete function...");
    }

    //Deletes one whole node ...
    fn delete_node(&self, doc_node: &Arc<DocumentNode>) {
        delete_node(doc_node);
    }

    fn isolate(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        return Ok(doc_node.clone());
    }

    //Yeah pictures do not merge
    fn try_merge(&self, _cursor: &Cursor, _block_node: &Arc<DocumentNode>) -> Result<()> {
        Ok(())
    }
}
