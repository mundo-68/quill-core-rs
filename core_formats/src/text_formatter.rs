// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::format_const::NAME_TEXT;
use crate::util::node_morph::{
    delete_node, delete_text, split_text_at_cursor, try_3_way_merge_text,
};
use crate::{t_attributes, t_formats};
use anyhow::Result;
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use dom::dom_element::DomElement;
use dom::dom_text::DomText;
use log::error;
use node_tree::cursor::Cursor;
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::{insert_at_index, unlink};
use node_tree::format_trait::FormatTait;
use std::sync::Arc;

#[allow(dead_code)]
const NBSP: &str = "\u{00a0}";

//If there is no format defined, but there are attributes to add, only then we will create a SPAN
static SPAN: &str = "SPAN";

//Just an empty struct to allow implementation of the trait, and give it a name
#[derive(Default)]
pub struct TextFormat {}

impl TextFormat {
    pub fn new() -> Self {
        initialise();
        TextFormat {}
    }
}

pub(crate) fn initialise() {
    t_attributes::initialise();
    t_formats::initialise();
}

/// # TextFormat
///
/// This will apply to any `DeltaOperation` unless it is a basic paragraph, and contains a `\n`
/// character. Check the `TextFormatter::apply()` function implementation in this trait for details.
///
/// To get most of of this formatter, it should be the last formatter registered in the
/// `Registry`. If not, it will "grab" any delta it can get, and other formats later in the
/// registry will not be able to respond to anything that comes by ...
impl FormatTait for TextFormat {
    fn create(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let attr = operation.get_attributes();
        let txt = operation.insert_value().str_val()?;

        if t_formats::has_text_fomat(attr) {
            apply(txt, attr, &formatter)
        } else if t_attributes::has_text_attributes(attr) {
            apply_attributed_only(txt, attr, &formatter)
        } else {
            apply_text_only(txt, attr, &formatter)
        }
    }

    //Transforms a <P> text element in to some other block element
    fn format_name(&self) -> &'static str {
        NAME_TEXT
    }

    //fn is_block_format(&self, _delta: &DeltaOperation) -> bool { false }

    fn is_text_format(&self) -> bool {
        true
    }

    fn block_remove_attr(&self) -> Attributes {
        panic!("Hey you called text_formatter::block_remove_attr() on the TextFormatter format-trait implementation.");
    }

    fn applies(&self, delta: &DeltaOperation) -> Result<bool> {
        if delta.insert_value().is_string() && delta.insert_value().str_val()?.eq("\n") {
            error!("text_formatter::applies() --> detected block format");
            return Ok(false);
        }
        let attr = delta.get_attributes();
        if attr.is_empty() {
            return Ok(true);
        } //insert text without attributes ...
        if t_formats::has_text_fomat(attr) || t_attributes::has_text_attributes(attr) {
            return Ok(true);
        }
        Ok(false)
    }

    fn apply_line_attributes(
        &self,
        doc_node: &Arc<DocumentNode>,
        attr: &Attributes,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let index = doc_node.my_index_as_child().unwrap();
        assert!(doc_node.get_formatter().is_text_format());

        let old_op = doc_node.get_operation();
        let old_text = old_op.insert_value().str_val()?;
        let op = DeltaOperation::insert_attr(old_text, attr.clone());
        let new_doc_node = self.create(op, formatter)?;

        let parent = doc_node.get_parent().unwrap();
        unlink(&parent, doc_node);
        insert_at_index(&parent, index, new_doc_node.clone());
        Ok(new_doc_node)
    }

    fn drop_line_attributes(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        assert!(doc_node.get_formatter().is_text_format());
        let index = doc_node.my_index_as_child().unwrap();
        let format = doc_node.get_formatter();
        let old_op = doc_node.get_operation();
        let old_text = old_op.insert_value().str_val()?;
        let op = DeltaOperation::insert_attr(old_text, Attributes::default());
        let new_doc_node = self.create(op, format)?;

        let parent = doc_node.get_parent().unwrap();
        unlink(&parent, doc_node);
        insert_at_index(&parent, index, new_doc_node.clone());
        Ok(new_doc_node)
    }

    fn split_leaf(&self, cursor: &Cursor) -> Result<()> {
        split_text_at_cursor(cursor)?;
        Ok(())
    }

    fn is_same_format(&self, left: &Arc<DocumentNode>, right: &Arc<DocumentNode>) -> bool {
        left.get_operation()
            .get_attributes()
            .is_equal(right.get_operation().get_attributes())
    }

    fn block_transform(
        &self,
        _cursor: &Cursor,
        _block_node: &Arc<DocumentNode>,
        _delta: DeltaOperation,
        _new_transducer: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        panic!("Hey you called text_formatter::transform() on the TextFormatter format-trait implementation.");
    }

    fn un_block_transform(
        &self,
        _cursor: &Cursor,
        _block_node: &Arc<DocumentNode>,
    ) -> Result<Arc<DocumentNode>> {
        panic!("Hey you called text_formatter::un_block_transform() on the TextFormatter format-trait implementation.");
    }

    //length shall be equal or less than own length
    fn delete_leaf_segment(
        &self,
        doc_node: &Arc<DocumentNode>,
        at: usize,
        length: usize,
    ) -> Result<()> {
        assert!(doc_node.op_len() > length);
        delete_text(doc_node, at, length)?;
        Ok(())
    }

    //Deletes one whole node ...
    fn delete_node(&self, doc_node: &Arc<DocumentNode>) {
        delete_node(doc_node);
    }

    fn isolate(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        Ok(doc_node.clone())
    }

    fn try_merge(&self, cursor: &Cursor, _block_node: &Arc<DocumentNode>) -> Result<()> {
        try_3_way_merge_text(cursor)?;
        Ok(())
    }
}

/// Method to handle text without any formatting or attributes...
fn apply_text_only(
    txt: &str,
    attr: &Attributes,
    formatter: &Arc<dyn FormatTait + Send + Sync>,
) -> Result<Arc<DocumentNode>> {
    let operation = DeltaOperation::insert_attr(txt, attr.clone());
    let text = DomText::new(txt);
    let dd = DocumentNode::new_text(text, formatter.clone());
    dd.set_operation(operation);
    let doc_node = Arc::new(dd);
    Ok(doc_node)
}

/// Formats are things that go in to the `<html_element>` attributes like so:
///  - <SPAN font="mono">text</SPAN>
///
/// The `attributes` can be added to `<I>`, or `<STRONG>` but not a "naked" text element.
/// In case it is an "naked" text element, we add a `<SPAN>`, and insert the attributes there.
#[inline]
fn apply_attributed_only(
    txt: &str,
    attr: &Attributes,
    formatter: &Arc<dyn FormatTait + Send + Sync>,
) -> Result<Arc<DocumentNode>> {
    let operation = DeltaOperation::insert_attr(txt, attr.clone());
    let span = DomElement::new(SPAN);
    t_attributes::apply_text_attributes(&span, attr)?;
    let text = DomText::new(txt);
    span.append_child(text.node());

    let dd = DocumentNode::new_element(span, formatter.clone());
    dd.set_operation(operation);
    let doc_node = Arc::new(dd);
    Ok(doc_node)
}

/// This method handles attributed and formatted text.
///
/// Attributed text are HTML elements that wrap some `text` element like so:
///  - `<bold>text</bold>`
///
/// Multiple attributes wrap each other like so:
///  - `<I><bold>text</bold></I>`
///
/// Formats are things that go in to the `<html_element>` attributes like so:
///  - <SPAN font="mono">text</SPAN>
///
/// The `attributes` can be added to `<I>`, or `<STRONG>` but not a "naked" text element.
/// In case it is an "naked" text element, we add a `<SPAN>`, and insert the attributes there.
///
#[inline]
fn apply(
    txt: &str,
    attr: &Attributes,
    formatter: &Arc<dyn FormatTait + Send + Sync>,
) -> Result<Arc<DocumentNode>> {
    let operation = DeltaOperation::insert_attr(txt, attr.clone());
    let text_node = DomText::new(txt);
    let dd = DocumentNode::new_text(text_node, formatter.clone());
    dd.set_operation(operation);
    let mut doc_node = Arc::new(dd);
    doc_node = t_formats::apply_text_formats(&doc_node, attr);

    //Apply text attributes for the format HTML element
    t_attributes::apply_text_attributes(doc_node.get_dom_element().unwrap(), attr)?;

    Ok(doc_node)
}
