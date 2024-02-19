// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::util::block_format;
use crate::P_FORMAT;
use anyhow::Result;
use delta::attributes::{compose, Attributes};
use delta::operations::DeltaOperation;
use log::error;
use node_tree::cursor::Cursor;
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::{append, insert_before, unlink};
use node_tree::format_trait::FormatTait;
use std::sync::Arc;

/// # block_transform()
///
/// Transforms a node from one block format into another.<br>
/// The new block is inserted in the parent of the original block.
///
/// We should not have to update the cursor, neither should the cursor point to the block
/// being updated, unless ... it is an empty block node!
///
/// Pre conditions:
///  - the block_node has a parent
///  - the format to transform into is block format
#[inline(always)]
pub fn block_transform(
    block_node: &Arc<DocumentNode>,
    delta: DeltaOperation,
    format: Arc<dyn FormatTait + Send + Sync>,
    cursor: &Cursor,
) -> Result<Arc<DocumentNode>> {
    assert!(block_node.get_parent().is_some());
    assert!(!format.is_text_format());

    let update_cursor = cursor.get_doc_node().eq(block_node);

    let new_block = format.create(delta, format.clone())?;

    let parent = block_node.get_parent().unwrap();
    insert_before(&parent, block_node, new_block.clone());
    unlink(&parent, block_node);

    //Empty block node if cursor points to blocknode
    if update_cursor {
        cursor.set_at_no_retain_update(&new_block, 0);
        return Ok(new_block);
    }

    //the cursor should never point to a block if it is not empty ... right?
    assert!(!cursor.get_doc_node().eq(block_node));

    //Non empty block node
    let children = block_node.get_children();
    for i in 0..children.len() {
        let c = children.get(i).unwrap();
        unlink(block_node, c);
        append(&new_block, c.clone());
    }
    Ok(new_block)
}

/// # un_block_transform()
///
/// Removes the formatting from a block. Bringing it back to a `normal` paragraph block.
///
/// We should not have to update the cursor, neither should the cursor point to the block
/// being updated, unless ... it is an empty block node!
pub fn un_block_transform(
    block_node: &Arc<DocumentNode>,
    cursor: &Cursor,
) -> Result<Arc<DocumentNode>> {
    assert!(block_node.get_parent().is_some());
    assert!(!block_node.is_text());

    let update_cursor = cursor.get_doc_node().eq(block_node);

    let parent = block_node.get_parent().unwrap();

    let op = DeltaOperation::insert_attr("\n", block_node.get_operation().get_attributes().clone());
    let new_block = P_FORMAT.create(op, P_FORMAT.clone())?;
    insert_before(&parent, block_node, new_block.clone());
    unlink(&parent, block_node);

    //Empty block node if cursor points to blocknode
    if update_cursor {
        cursor.set_at_no_retain_update(&new_block, 0);
        return Ok(new_block);
    }

    let children = block_node.get_children();
    for i in 0..block_node.child_count() {
        let c = children.get(i).unwrap();
        unlink(block_node, c);
        append(&new_block, c.clone());
    }
    Ok(new_block)
}

/// # apply_attributes()
///
/// applies the attributes to a given doc-node
pub fn apply_attributes(
    doc_node: &Arc<DocumentNode>,
    attr: &Attributes,
) -> Result<Arc<DocumentNode>> {
    let element = doc_node.get_dom_element().unwrap();
    let attributes = compose(attr, doc_node.get_operation().get_attributes(), false);
    error!("BLOCK::apply_attributes(): {:?}", attributes);

    block_format::apply(element, &attributes)?;
    let operation = DeltaOperation::insert_attr(
        doc_node.get_operation().insert_value().clone(),
        attr.clone(),
    );
    doc_node.set_operation(operation);
    Ok(doc_node.clone())
}

/// # drop_attributes()
///
/// drops ALL the attributes from a given doc-node
pub fn drop_attributes(doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
    let element = doc_node.get_dom_element().unwrap();
    let attr = Attributes::default();
    block_format::apply(element, &attr)?;
    let operation = DeltaOperation::insert_attr(
        doc_node.get_operation().insert_value().clone(),
        attr.clone(),
    );
    doc_node.set_operation(operation);
    Ok(doc_node.clone())
}
