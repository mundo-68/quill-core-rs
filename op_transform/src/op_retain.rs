// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::auto_soft_break::AutomaticSoftBreak;
use crate::registry::Registry;
use anyhow::Result;
use core_formats::util::node_morph::split_text_node;
use delta::attributes::{compose, Attributes};
use delta::operations::DeltaOperation;
use log::error;
use node_tree::cursor::{Cursor, CursorLocation};
use node_tree::doc_node::DocumentNode;
use node_tree::tree_traverse::{first_node, next_node_non_zero_length};
use std::sync::{Arc, RwLockReadGuard};

/// # Retain()
///
/// Either moves the cursor, or updates the attributes.
///
/// Retain index is simply current retain index + length of the delta operation
pub fn retain(
    cursor: &Cursor,
    delta: &DeltaOperation,
    registry: &RwLockReadGuard<'static, Registry>,
) -> Result<()> {
    let retain = cursor.get_retain_index();
    cursor.set_retain_index(retain + delta.op_len());

    if delta.get_attributes().is_empty() {
        retain_length(cursor, delta.op_len());
    } else {
        retain_attributed(
            cursor,
            delta.op_len(),
            delta.get_attributes().clone(),
            registry,
        )?;
    }
    Ok(())
}

/// # set_cursor_selection()
///
/// Creates a selection for a cursor starting at the current start position,
/// then ends a the given length.
///
/// Rationale for this function: <br>
/// Well it is not possible to transform the document node tree without changing a
/// node. Hence if we try to restore a selection in the document, then we can not
/// just copy the cursor, and restore it later, because the "remembered" cursor
/// may point to doc nodes which do not exist anymore ...
pub fn set_cursor_selection(cursor: &Cursor, retain_index: usize, selection_length: usize) {
    //set the start
    //----------------------------------------
    let node = first_node(&cursor.get_doc_node());
    if node.get_formatter().is_text_format() {
        cursor.set_before(&node);
    } else {
        cursor.set_at(&node, 0);
    };
    cursor.set_retain_index(retain_index);
    retain_length(cursor, retain_index);

    //set the end
    //----------------------------------------
    if selection_length > 0 {
        let c = cursor.clone();
        retain_length(&c, selection_length);
        //we need start, since retain moves the start position of the cursor
        cursor.set_select_stop(c.get_select_start());
    } else {
        cursor.set_select_stop(CursorLocation::None);
    }
}

/// # retain_length()
///
/// Moves the cursor the the next location, starting from the current cursor location.
fn retain_length(cursor: &Cursor, retain_len: usize) {
    let (mut dn, mut rtn) = match cursor.get_location() {
        CursorLocation::After(dn) => match next_node_non_zero_length(&dn) {
            None => {
                return;
            }
            Some(doc_node) => (doc_node, retain_len),
        },
        CursorLocation::Before(dn) => (dn, retain_len),
        CursorLocation::At(dn, idx) => {
            if !dn.get_formatter().is_text_format() {
                //pointing to an empty block
                (dn, retain_len)
            } else {
                //We do not want to split, nor have complex logic here... so we retain a bit more ...
                (dn, retain_len + idx)
            }
        }
        _ => {
            panic!("retain(): Cursor at NONE position")
        }
    };
    //post condition: Cursor is "before" some node to be retained and nothing has been retained yet

    //start consuming whole DocumentNode, until we can not consume whole nodes anymore
    //only then consume the remainder of the active DocumentNode
    while rtn > 0 {
        let ol = dn.get_operation().op_len();
        if rtn >= ol {
            rtn = rtn - ol;
            dn = match next_node_non_zero_length(&dn) {
                None => {
                    cursor.set_cursor_to_doc_node_edge(&dn, false);
                    error!("retain_length() - We try to retain but no next node.");
                    return;
                }
                Some(doc_node) => {
                    if rtn == 0 {
                        cursor.set_cursor_to_doc_node_edge(&doc_node, true);
                        return;
                    }
                    doc_node
                }
            };
        } else {
            cursor.set_at_no_retain_update(&dn, rtn);
            return;
        }
    }
}

/// # retain_attributed()
///
/// Change the attributes of a number of DocumentNodes over the given retain_length.
///
/// So what happens to a block node:
/// - it may change in to a new block node
/// - it may gain or lose some formatting attributes
/// BUT it will not change in to a text node!!  --> for that we need to delete the `\n` in the text
/// content. Which goes against the definition of a retain
///
/// And for a leaf node ... it may indeed gain or lose some formatting.
///
fn retain_attributed(
    cursor: &Cursor,
    retain_len: usize,
    attr: Attributes,
    registry: &RwLockReadGuard<'static, Registry>,
) -> Result<()> {
    let (mut dn, mut rtn) = match cursor.get_location() {
        CursorLocation::After(dn) => {
            match next_node_non_zero_length(&dn) {
                // the last element of a document is <p> so there is always a next one :-)
                None => {
                    return Ok(());
                }
                Some(doc_node) => (doc_node, retain_len),
            }
        }
        CursorLocation::Before(dn) => (dn, retain_len),
        CursorLocation::At(dn, idx) => {
            if !dn.get_formatter().is_text_format() {
                (dn, retain_len) //pointing to an empty block
            } else {
                let right_node = split_text_node(&dn, idx)?;
                (right_node, retain_len)
            }
        }
        _ => {
            panic!("retain(): Cursor at NONE position")
        }
    };
    //post condition: Cursor is "before" some node to be retained and nothing has been retained yet

    while rtn > 0 {
        //dn = dn.get_formatter().isolate(&dn);
        let ol = dn.get_operation().op_len();
        if rtn > ol {
            if dn.get_formatter().is_text_format() {
                dn = retain_text_format(&dn, &attr, &registry)?;
            } else {
                dn = retain_block_format(&dn, &attr, &cursor, &registry)?
            }
            if let Some(next) = next_node_non_zero_length(&dn) {
                //more loops to do and more nodes to consume
                cursor.set_cursor_to_doc_node_edge(&next, true);
                dn = next;
                rtn = rtn - ol;
            } else {
                //more retains requested BUT no nodes to consume
                cursor.set_after_no_retain_update(&dn);
                error!(
                    "(1) Document seems empty, and we are requested to still retain: {} characters",
                    rtn - ol
                );
                return Ok(());
            }
        } else if rtn == ol {
            if dn.get_formatter().is_text_format() {
                dn = retain_text_format(&dn, &attr, &registry)?;
            } else {
                dn = retain_block_format(&dn, &attr, &cursor, &registry)?;
            }
            rtn = 0;
            //stopped at the next loop start ...but first update the cursor
            if let Some(next) = next_node_non_zero_length(&dn) {
                //more loops to do and more nodes to consume
                cursor.set_cursor_to_doc_node_edge(&next, true);
                dn = next;
            } else {
                //more retains requested BUT no nodes to consume
                cursor.set_after_no_retain_update(&dn);
                error!(
                    "(2) Document seems empty, and we are requested to still retain: {} characters",
                    rtn - ol
                );
                return Ok(());
            }
        } else if rtn > 0 {
            let right = split_text_node(&dn, rtn)?;
            retain_text_format(&dn, &attr, &registry)?; //this can only be text format ...
            cursor.set_cursor_to_doc_node_edge(&right, true);
            rtn = 0;
        }
        cursor
            .get_doc_node()
            .get_formatter()
            .try_merge(cursor, &cursor.get_doc_node())?;
    }
    Ok(())
}

/// # retain_block_format()
///
/// do the retain operation for a block format
///  - the changed current document node pointed to is returned, allowing proper cursor handling
fn retain_block_format(
    doc_node: &Arc<DocumentNode>,
    attr: &Attributes,
    cursor: &Cursor,
    registry: &RwLockReadGuard<'static, Registry>,
) -> Result<Arc<DocumentNode>> {
    let new_block = doc_node
        .get_formatter()
        .un_block_transform(&cursor, &doc_node)?;
    let operation = doc_node.get_operation(); // old block format removed from operation ...

    let val = operation.insert_value();
    let attr = compose(operation.get_attributes(), attr, false);
    //error!("retain_block_format() attributes =  {:?}", &attr);
    let operation = DeltaOperation::insert_attr(val.clone(), attr);

    let format = registry.block_format(&operation)?.clone();
    let doc_node = format.block_transform(cursor, &new_block, operation, format.clone())?;
    if doc_node.child_count() == 0 {
        AutomaticSoftBreak::insert(&doc_node)?;
    }
    Ok(doc_node)
}

/// # retain_text_format()
///
/// do the retain operation for a text format
///  - the changed current document node pointed to is returned, allowing proper cursor handling
fn retain_text_format(
    doc_node: &Arc<DocumentNode>,
    attr: &Attributes,
    registry: &RwLockReadGuard<'static, Registry>,
) -> Result<Arc<DocumentNode>> {
    let operation = doc_node.get_operation();

    let new_block = doc_node.get_formatter().drop_line_attributes(&doc_node)?;
    let attr = compose(operation.get_attributes(), attr, false);

    let format = registry.line_format(&operation)?.clone();
    let doc_node = format.apply_line_attributes(&new_block, &attr, format.clone())?;
    Ok(doc_node)
}
