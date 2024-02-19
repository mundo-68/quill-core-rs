// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::auto_soft_break::AutomaticSoftBreak;
use crate::error::Error::{
    CanNotFindNextBlock, DeleteOperationOnEmptyDocument, DeletingLastBlock,
    UnexpectedCursorPosition,
};
use anyhow::Result;
use core_formats::util::node_morph::split_text_node;
use node_tree::cursor::{Cursor, CursorLocation};
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::{insert_at_index, unlink};
use node_tree::tree_traverse::{next_block, next_node_non_zero_length, prev_node};
use std::sync::Arc;

/// # delete()
///
/// Deletes a number of characters from the document
/// Deleting a character happens "after" the cursor so
/// there is no change in cursor location. All cursor
/// updates will use the one without retain update ...
pub fn delete(cursor: &Cursor, delete_len: usize) -> Result<()> {
    let (mut dn, mut del) = match cursor.get_location() {
        CursorLocation::After(dn) => match next_node_non_zero_length(&dn) {
            None => {
                return Ok(());
            }
            Some(doc_node) => (doc_node, delete_len),
        },
        CursorLocation::Before(dn) => (dn, delete_len),
        CursorLocation::At(dn, idx) => {
            if !dn.get_formatter().is_text_format() {
                //pointing to an empty block
                assert_eq!(idx, 0);
                (dn, delete_len)
            } else {
                let right_node = split_text_node(&dn, idx)?;
                cursor.set_after_no_retain_update(&dn);
                (right_node, delete_len)
            }
        }
        CursorLocation::None => {
            return Err(UnexpectedCursorPosition {
                pos: "None".to_string(),
            }
            .into());
        }
    };
    //post condition: Cursor is "before" some node to be deleted and nothing has been deleted yet

    //error!( "op_transform::delete() - format name = {}", dn.get_formatter().format_name());

    while del > 0 {
        let ol = dn.get_operation().op_len();
        if del > ol {
            if let Some(next) = next_node_non_zero_length(&dn) {
                //more loops to do and more nodes to consume
                delete_document_node(&dn)?;
                dn = next;
                del = del - ol;
            } else {
                //more deletes requested BUT no nodes to consume
                let found = find_left_node_and_set_cursor(&dn, &cursor);
                delete_document_node(&dn)?;
                if !found {
                    return Err(DeleteOperationOnEmptyDocument.into());
                }
                //No need for merging text, there is no text after to merge ...
                return Ok(());
            }
        } else if del == ol {
            if let Some(next) = next_node_non_zero_length(&dn) {
                //last delete action, and next nodes to the right found
                delete_document_node(&dn)?;
                cursor.set_cursor_to_doc_node_edge(&next, true);
                dn = next;
            } else {
                let found = find_left_node_and_set_cursor(&dn, &cursor);
                delete_document_node(&dn)?;
                if !found {
                    return Err(DeleteOperationOnEmptyDocument.into());
                }
            }
            del = 0;
            //stopped at the next loop start ...
        } else if del > 0 {
            dn.get_formatter().delete_leaf_segment(&dn, 0, del)?;
            cursor.set_before_no_retain_update(&dn);
            del = 0;
        }
    }
    let node = cursor.get_doc_node();
    node.get_formatter().try_merge(cursor, &node)?;

    //We should not stick the cursor to a DOC node with length 0
    assert!(cursor.get_doc_node().op_len() > 0);
    Ok(())
}

/// # find_left_node_and_set_cursor()
///
/// Finds previous node to put the cursor after the delete action has consumed all right
/// hand side `DocumentNodes`
///
/// input is the current to be deleted `DocumentNode`
/// returns true if a proper cursor position has been found; else the document is empty ...
fn find_left_node_and_set_cursor(doc_node: &Arc<DocumentNode>, cursor: &Cursor) -> bool {
    if let Some(prev) = prev_node(&doc_node) {
        if prev.get_formatter().is_text_format() {
            cursor.set_after_no_retain_update(&prev);
        } else if prev.child_count() == 0 {
            cursor.set_at_no_retain_update(&prev, 0);
        } else {
            let children = prev.get_children();
            let last = children.get(prev.child_count() - 1).unwrap();
            cursor.set_after_no_retain_update(&last);
        }
        return true;
    }
    return false;
}

/// # delete_document_node()
///
/// What if we delete a block_node?
/// - then we first make it empty, and then delete the block node
fn delete_document_node(doc_node: &Arc<DocumentNode>) -> Result<()> {
    let Some(parent) = next_block(&doc_node) else {
        return if !doc_node.is_text() {
            // Test for last block ... which we never shall delete
            Err(DeletingLastBlock.into())
        } else {
            Err(CanNotFindNextBlock.into())
        };
    };

    if doc_node.get_formatter().is_text_format() {
        doc_node.get_formatter().delete_node(&doc_node);
        //If we deleted ALL children of a parent then we should add a line break
        if parent.is_empty_block() {
            AutomaticSoftBreak::insert(&parent)?;
        }
    } else if doc_node.child_count() == 0 {
        // we are a block ... but empty remove the block
        doc_node.get_formatter().delete_node(&doc_node);
    } else {
        // we are deleting the block, but not the children --> merge children with next block
        // `parent` should now point to the NEXT block to merge the children into
        if parent.is_empty_block() {
            AutomaticSoftBreak::remove(&parent)?;
        }

        let block_parent = doc_node.get_parent().unwrap();
        unlink(&block_parent, &doc_node);

        let children = doc_node.get_children();
        for child in children.into_iter().rev() {
            unlink(&doc_node, &child);
            insert_at_index(&parent, 0, child);
        }
        assert_eq!(doc_node.child_count(), 0);
        doc_node.get_formatter().delete_node(&doc_node);
    }
    Ok(())
}
