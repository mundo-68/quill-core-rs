// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::util::string_util::StringUtils;
use crate::TEXT_FORMAT;
use anyhow::Result;
use delta::operations::DeltaOperation;
use delta::types::ops_kind::OpKind;
use dom::dom_text::find_dom_text;
use node_tree::cursor::{Cursor, CursorLocation};
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::{append, insert_after, insert_at_index, unlink};
use node_tree::format_trait::FormatTait;
use node_tree::tree_traverse::{next_sibling, prev_sibling};
///==============================================================================================
/// Tree node morphing:
/// Nodes will deform, by adding/removing text, or splitting the text
/// FIXME: Can we change these to a node trait, so that we can use "self" and node.split ... or so?
///==============================================================================================
use std::sync::Arc;

/// Helper location enum
enum Location {
    Before,
    After,
    At,
}

/// # split_text_at_cursor()
///
/// Split ONLY the leaf node to which the cursor points.
///
/// Examples below use:
/// - `[]` to show individual `text` HTML DOM elements.<br>
/// - the location of the cursor is shown as '{*}`
///
/// Example with plain text, splits in to 2 `<text>` nodes:
/// `<P>[Hel{*}lo world]</P>' <br>
/// results in: <br>
/// `<P>\[Hel]\[{*}lo world]</P>`
///
/// Example with more complex text element, like `<a>` splits into 2x `<a>` element:<br>
/// `<a href="">hel{*}lo world</a>`<br>
/// results in: <br>
/// `<a href="">hel</a>{*}<a href="">lo world</a>`
///
/// Post condition: Cursor position is BEFORE the right hand side created node.
pub fn split_text_at_cursor(cursor: &Cursor) -> Result<()> {
    match cursor.get_location() {
        CursorLocation::At(doc_node, index) => {
            //empty block node?
            if index == 0 {
                return Ok(());
            }
            //we are a text node!
            let right = split_text_node(&doc_node, index)?;
            cursor.set_before_no_retain_update(&right);
        }
        _ => {
            //Nothing to split when we are `BEFORE` or `AFTER` with the cursor
        }
    }
    Ok(())
}

/// # split_text_and_block_at_cursor()
///
/// Splits both the leaf node to which the cursor points, and its **immediate** parent
/// into two halves.
///
/// It results in creating a new block node which format is identical to the parent
/// block node. The `left` parent node contains all leaf nodes before the cursor.
/// The `right` one all remaining ones.
///
/// If there are no leaf nodes to split then an EMPTY block node is created.<br>
///
/// Nothing is returned as we may create an empty left or right hand node. This would give
/// rise to un-determined return value of either an empty `block node`, or some `text
/// node` being returned.
///
/// Post condition:
///  - Normally the cursor position is `BEFORE` the right hand leaf node
///  - When the right hand block is empty, then the cursor is `AT[0]` the right hand block node
#[inline(always)]
pub fn split_text_and_block_at_cursor(cursor: &Cursor) -> Result<()> {
    let leaf_node = cursor.get_doc_node();
    let leaf_format = leaf_node.get_formatter();
    let parent = leaf_node.get_parent().unwrap();

    //if we are not a leaf, then the document node is empty
    //In this case this means we can also copy the operation in the new block
    if !leaf_format.is_text_format() {
        let right_node = insert_empty_block_node_after_cursor(cursor)?;
        cursor.set_at_no_retain_update(&right_node, 0);
        return Ok(());
    }

    //So we are a leaf ...
    match cursor.get_location() {
        CursorLocation::At(_doc_node, _at) => {
            assert!(leaf_format.is_text_format());
            //use the `FormatTrait::split_leaf()`, since the format might change the default behaviour
            //use split_text_at_cursor() for normal use in the trait definitions
            leaf_format.split_leaf(cursor)?;
            split_block_at_cursor(cursor)?;
        }
        CursorLocation::Before(leaf_node) => {
            split_block_before_child(&parent, &leaf_node)?;
        }
        CursorLocation::After(leaf_node) => {
            if let Some(next) = next_sibling(&leaf_node) {
                split_block_before_child(&parent, &next)?;
                cursor.set_before_no_retain_update(&next);
            } else {
                let right = parent.get_formatter().clone_doc_node(&parent)?;
                insert_after(&parent.get_parent().unwrap(), &parent, &right);
                cursor.set_at_no_retain_update(&right, 0);
            }
        }
        CursorLocation::None => {
            panic!("split_block(): cursor position is NONE");
        }
    }
    Ok(())
}

pub fn split_block_at_cursor(cursor: &Cursor) -> Result<()> {
    let format = cursor.get_doc_node().get_formatter();

    match cursor.get_location() {
        CursorLocation::At(doc_node, index) => {
            if index == 0 {
                assert!(!format.is_text_format());
                let operation = doc_node.get_operation();
                let clone = format.create(operation, format.clone())?;
                insert_after(&doc_node.get_parent().unwrap(), &doc_node, &clone.clone());
                cursor.set_at_no_retain_update(&clone, 0);
            } else {
                panic!(
                    "node_morph::split_block_at_cursor() --> received cursor pointing to text node"
                )
            }
        }
        CursorLocation::After(doc_node) => {
            if let Some(next_node) = next_sibling(&doc_node) {
                split_block_before_child(&doc_node.get_parent().unwrap(), &next_node)?;
            } else {
                let right = insert_empty_block_node_after_cursor(cursor)?;
                cursor.set_at_no_retain_update(&right, 0);
            }
        }
        CursorLocation::Before(doc_node) => {
            split_block_before_child(&doc_node.get_parent().unwrap(), &doc_node)?;
        }
        CursorLocation::None => {}
    }
    Ok(())
}

/// # insert_empty_block_node_after_cursor()
///
/// Helper function to append a next empty block node to a parent.<br>
/// The block format is copied from the first parent which is a block format.
///
/// When the cursor is pointing to an empty block node, then the empty block node
/// is cloned, and appended to its parent.
///
/// Examples:
///  `<P>[he{*}llo]</P><H1>world</H1>` <br>
/// results in <br>
/// `<P>[he{*}llo]</P><P></P><H1>world</H1>`
///
///
///  `<P>[hello{*}]</P><H1>world</H1>` <br>
/// results in <br>
/// `<P>[hello{*}]</P><P></P><H1>world</H1>`
///
///
///  `<P>{*}</P><H1>world</H1>` <br>
/// results in <br>
/// `<P>{*}</P><P></P><H1>world</H1>`
///
/// Note: This example changes the block node from `<P>` to `<H1>` it is not for this module
/// to change the bock type. So a standard `<P>` paragraph is added. See e.g. `op_transform::op_insert()`
/// for more details ...
///
/// Return value points to the added block node
pub fn insert_empty_block_node_after_cursor(cursor: &Cursor) -> Result<Arc<DocumentNode>> {
    let doc_node = cursor.get_doc_node();
    let format = doc_node.get_formatter();
    let parent = if format.is_text_format() {
        doc_node.get_parent().unwrap()
    } else {
        doc_node
    };

    let p_format = parent.get_formatter();
    let p_op = parent.get_operation();
    let next_node = p_format.create(p_op, p_format.clone())?;
    insert_after(&parent.get_parent().unwrap(), &parent, &next_node);
    Ok(next_node)
}

/// # try_3_way_merge_text()
///
/// Merges text between document nodes, in the same parent. We do a 3 way merge,
/// checking the sibling left, and right of the node pointed to by the cursor.
/// If either one is eligible for merging, then it is merged.
///
/// If there is nothing to be merged, then this is no error. This is denoted
/// by the "try_" in the function name.
///
/// It is assumed that the node right `before` the cursor is the one that
/// is the pivot.
///
/// Post condition: the cursor position is not changed, but may be `at` some
/// position, due to the merging.
#[inline(always)]
pub fn try_3_way_merge_text(cursor: &Cursor) -> Result<()> {
    let mut location = Location::Before;
    let mut position = 0;
    let dn = match cursor.get_location() {
        CursorLocation::Before(doc_node) => {
            if let Some(prev) = prev_sibling(&doc_node) {
                location = Location::After;
                position = prev.op_len();
                prev //--> this is most likely the one changed last
            } else {
                doc_node //if we are the first in a parent, we use that
            }
        }
        CursorLocation::After(doc_node) => {
            location = Location::After;
            position = doc_node.op_len();
            doc_node //--> this is most likely the one changed last
        }
        CursorLocation::At(doc_node, index) => {
            if doc_node.op_len() == 0 {
                panic!("node_morph::try_3_way_merge_text() - called for a empty block_node")
                //return
            }
            location = Location::At;
            position = index;
            doc_node
        }
        CursorLocation::None => {
            return Ok(());
        }
    };
    //Post condition: We have a starting doc_node.
    //error!("try_3_way_merge_text; middle doc node = {}\n", &dn);

    //start with the next text block since the merge_text_node() will never destroy it
    let next = next_sibling(&dn);
    if let Some(n) = next {
        if dn.get_operation().get_attributes() == n.get_operation().get_attributes() {
            //error!("try_3_way_merge_text; merge right = {}\n", &n);
            merge_text_node(&dn, &n)?;
            match location {
                Location::Before => {
                    panic!("ERROR: This case can never happen!!")
                    //cursor.set_before_no_retain_update(&dn);
                }
                _ => {
                    location = Location::At;
                    cursor.set_at_no_retain_update(&dn, position);
                }
            }
        }
    }
    let prev = prev_sibling(&dn);
    if let Some(p) = prev {
        if dn.get_operation().get_attributes() == p.get_operation().get_attributes() {
            //error!("try_3_way_merge_text; merge left = {}\n", &p);

            let prev_len = p.op_len();
            merge_text_node(&p, &dn)?;
            match location {
                Location::Before => {
                    panic!("ERROR: This case can never happen!!")
                    //cursor.set_at_no_retain_update(&p, prev_len);
                }
                Location::After => {
                    cursor.set_after_no_retain_update(&p);
                }
                Location::At => {
                    cursor.set_at_no_retain_update(&p, prev_len + position);
                }
            }
        }
    }
    Ok(())
}

/// # try_3_way_merge_block()
///
/// Merges a block and its sibling text. We do a 3 way merge,
/// checking the sibling left, and right of the input block.
/// If either one is eligible for merging, then it is merged.
/// Only the blocks are merged, not the text.
///
/// Example; the text:
/// ```html
///    <UL>
///         <LI>Hello</LI>
///    </UL>
///     <UL>
///         <LI>sweet</LI>
///    </UL>
///    <UL>
///        <LI>world</LI>
///    </UL>
/// ```
///
/// Is merged into:
/// ```html
///     <UL>
///        <LI>Hello</LI>
///        <LI>sweet</LI>
///        <LI>world</LI>
///    </UL>
/// ```
/// Where `{*}` is the current cursor position.
///
/// If there is nothing to be merged, then this is no error. This is denoted
/// by the "try_" in the function name.
///
/// Post condition:
///  - the cursor position is not changed, but may be `at` some position, due to the merging.
///  - when there is a `prev` node to be merged, then the `middle_block` has unlinked after this call
///
/// FIXME: Remove in favour of the method in formats::list
#[inline(always)]
pub fn try_3_way_merge_block(middle_block: &Arc<DocumentNode>) -> Result<()> {
    //start with the next block since the merge_block_node() will never destroy it
    if let Some(next) = next_sibling(middle_block) {
        if next.get_formatter().is_same_format(&next, middle_block) {
            merge_block_node(middle_block, &next)?;
        }
    }

    if let Some(prev) = prev_sibling(middle_block) {
        if prev.get_formatter().is_same_format(&prev, middle_block) {
            merge_block_node(&prev, middle_block)?;
        }
    }
    Ok(())
}

/// # split_text_node()
///
/// Splits the text node, into 2.
///
/// Examples below use:
/// - `[]` to show individual `text` HTML DOM elements.<br>
/// - the location of the cursor is shown as '{*}`
///
/// Example:
/// `<P>\[ab{*}cde]</P>` <br>
/// results in <br>
/// `<P>\[ab][{*}cde]</P>`
///
/// We create the right hand side format from scratch. Here we want ONLY the
/// normal text part. Some formats create more complicated structures so a
/// trick is applied:
/// - crate the structure using the `TEXT_FORMAT`
/// - add the original format to the `DocumentNode` structure
///
/// Post condition: the cursor position is `BEFORE` the right hand leaf node
///
/// returns: Right hand text-node of the split
pub fn split_text_node(doc_node: &Arc<DocumentNode>, index: usize) -> Result<Arc<DocumentNode>> {
    let delta = doc_node.get_operation();
    let d_l = delta.op_len() - index;
    let len = delta.op_len();

    //Do not split such that the end result is empty
    //Cursor never points to index 0, or len, since that is reserved for `After` and `Before`
    assert_ne!(index, 0);
    assert!(index < len);

    let (left, right) = split_at(&delta, index)?;

    let text = find_dom_text(doc_node.get_html_node()).unwrap();
    text.delete_text(index, d_l);
    doc_node.set_operation(left);

    //create new right node with the remainder and insert in the parent
    let parent = doc_node.get_parent().unwrap();
    let orig_format = doc_node.get_formatter();
    let format: Arc<dyn FormatTait + Send + Sync> = TEXT_FORMAT.clone();
    let index = parent.get_child_index(doc_node).unwrap();
    let ret = format.create(right, orig_format.clone())?;
    insert_at_index(&parent, index + 1, ret.clone());
    Ok(ret)
}

/// # split_at()
///
/// Split one DeltaOperation into 2 by splitting at a certain index location.
/// Both Delta operation will have the same Attributes.
///
/// Implementation Note: Assumes the operation is an insert() operation,
/// hence it contains a string as payload. This precondition is not checked / enforced.
pub fn split_at(op: &DeltaOperation, index: usize) -> Result<(DeltaOperation, DeltaOperation)> {
    let s = op.insert_value().str_val()?;
    let (l, r) = s.split_at(index);
    let mut left = DeltaOperation::insert(l);
    left.set_attributes(op.get_attributes().clone());
    let mut right = DeltaOperation::insert(r);
    right.set_attributes(op.get_attributes().clone());
    Ok((left, right))
}

/// # merge_text_node()
///
/// Merges 2 text nodes.<br>
/// The left doc node will contain the merged result.
/// The right doc node is dropped from the tree.
///
/// Implementation note:<br>
/// We assume the caller has checked that both are compatible in attributes.
/// If this is not the case, then the attributes of the right node are lost.
pub fn merge_text_node(left: &Arc<DocumentNode>, right: &Arc<DocumentNode>) -> Result<()> {
    let delta = merge(&left.get_operation(), &right.get_operation())?;
    let text = find_dom_text(left.get_html_node()).unwrap();
    text.append_text(right.get_operation().insert_value().str_val()?);
    unlink(&right.get_parent().unwrap(), right);
    left.set_operation(delta);
    Ok(())
}

/// # merge()
///
/// Merges 2 DeltaOperations, the right operation is concatenated after the left operation.
///
/// Implementation Note: Assumes the operation is an insert() operation,
/// hence it contains a string as payload. This precondition is not checked / enforced.
fn merge(left: &DeltaOperation, right: &DeltaOperation) -> Result<DeltaOperation> {
    assert!(left.get_attributes().is_equal(right.get_attributes()));
    let ls = left.insert_value().str_val()?;
    let rs = right.insert_value().str_val()?;
    let op_s = [ls, rs].concat();
    let mut op = DeltaOperation::insert(op_s);
    op.set_attributes(left.get_attributes().clone());
    Ok(op)
}

/// # split_block_before_child()
///
/// Splits a block at the boundary of 2 children in the block.
///
/// The result is 2 blocks of same type, and attributes. Each having a set of children.
/// The child given as input, will be the first child of the right hand created parent
/// This left hand set may be empty, when the  child is chosen to be the first child.
///
/// The left hand parent is changed, as intended, but not removed from the parent.
/// Return value is: the right hand created parent,
pub fn split_block_before_child(
    block_left: &Arc<DocumentNode>,
    child: &Arc<DocumentNode>,
) -> Result<Arc<DocumentNode>> {
    let child_index = block_left.get_child_index(child).unwrap();

    let block_right = block_left.get_formatter().clone_doc_node(block_left)?;

    //Note: children is a CLONE of the array of children in the right hand node
    //For that reason we can remove children from the right parent in the loop below,
    //since it will not change the content of the `children` array
    let children = block_left.get_children();
    for i in child_index..children.len() {
        let c = children.get(i).unwrap();
        unlink(block_left, c);
        append(&block_right, c.clone());
    }

    let p_parent = block_left.get_parent().unwrap();
    insert_after(&p_parent, block_left, &block_right);

    Ok(block_right)
}

/// # merge_block_node()
///
/// Merges 2 block nodes in to one.
///
/// All children of the right hand block will be added to the left side block. Then the
/// right hand block will be deleted. No checks are done on the attributes, of the
/// blocks. So if they are not equal, then some formatting may be lost.
pub fn merge_block_node(left: &Arc<DocumentNode>, right: &Arc<DocumentNode>) -> Result<()> {
    //Note: children is a CLONE of the array of children in the right hand node
    //For that reason we can remove children from the right parent in the loop below,
    //since it will not change the content of the `children` array
    let children = right.get_children();
    for i in 0..children.len() {
        let c = children.get(i).unwrap();
        unlink(right, c);
        append(left, c.clone());
    }

    let parent = right.get_parent().unwrap();
    unlink(&parent, right);
    Ok(())
}

/// # delete_node()
///
/// Deletes one whole node.
///
/// Pre condition: If we are a block, then deleting the block should only happen for an empty block.
/// See `node_tree::cursor` module for more details
#[inline(always)]
pub fn delete_node(doc_node: &Arc<DocumentNode>) {
    assert!(
        (!doc_node.get_formatter().is_text_format() && doc_node.child_count() == 0)
            || doc_node.get_formatter().is_text_format(),
        "node_morph::delete_node(): block node child count = {:?} \nDeltaOperation = {:?}",
        doc_node.child_count(),
        doc_node.get_operation()
    );

    let parent = doc_node.get_parent().unwrap();
    unlink(&parent, doc_node);
}

/// # delete_text()
///
/// Deletes a substring in the text in a `DocumentNode`.
///
/// Pre conditions:
/// - text node contains at least 1 character after delete
/// - `at(i)`: cursor position points to first to be deleted character
/// - After `at(i) there are still at least `length` characters left
pub fn delete_text(doc_node: &Arc<DocumentNode>, at: usize, length: usize) -> Result<()> {
    assert!(doc_node.get_formatter().is_text_format());
    assert!(doc_node.op_len() > length); //Do not delete the WHOLE node this way, use delete node in that case
    assert!(doc_node.op_len() >= at + length); // Hey you are deleting more than my length

    //change DeltaOperation
    let mut op = doc_node.get_operation();
    delete_at(&mut op, at, length)?;
    doc_node.set_operation(op);

    //Change HTML DOM
    //We may have formatting ... so we look for the first "text child" starting from
    //the current DOM node.
    //
    //Example, with {*} showing current cursor, which points just before `doc_node`:
    //<P>{*}<b><em>hello world</b></em></P>
    doc_node.find_dom_text().delete_text(at, length);
    Ok(())
}

/// # insert_text()
///
/// Inserts a substring in the text in a `DocumentNode`.
///
/// Pre conditions:
/// - text node contains at least 1 character
/// - `at(i)`: cursor position points to the character before which the new text is insertedF
/// - After `at(i) there are still at least `length` characters left
pub fn insert_text(doc_node: &Arc<DocumentNode>, at: usize, txt: &str) -> Result<()> {
    let text = doc_node.find_dom_text();
    let mut op = doc_node.get_operation().clone();

    text.insert_text(at, txt);
    insert_at(&mut op, at, txt)?;
    doc_node.set_operation(op);
    Ok(())
}

/// # insert_at()
///
/// Inserts a string into a DeltaOperation, at the given index location.
/// Any content after the original content location is appended after the
/// to be inserted string
///
/// Implementation Note: Assumes the operation is an insert() operation,
/// hence it contains a string as payload. This precondition is not checked / enforced.
fn insert_at(op: &mut DeltaOperation, at: usize, s: &str) -> Result<()> {
    let txt = op.insert_value().str_val()?;
    let (l, r) = txt.split_at(at);
    let op_s = [l, s, r].concat();
    op.set_op_kind(OpKind::from(op_s));
    Ok(())
}

/// # delete_at()
///
/// Deletes the content of a DeltaOperation, starting at a certain index,
/// and deletes a given number of characters.
///
/// Implementation Note: Assumes the operation is an insert() operation,
/// hence it contains a string as payload. This precondition is not checked / enforced.
fn delete_at(op: &mut DeltaOperation, at: usize, len: usize) -> Result<()> {
    assert!(op.op_len() >= at + len);
    let s = op.insert_value().str_val()?;
    let (l, r) = s.split_at(at);
    let rr = r.substring(len, r.len());
    let res = [l, rr].concat();
    op.set_op_kind(OpKind::from(res));
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use delta::operations::DeltaOperation;

    #[test]
    fn insert_test() -> Result<()> {
        let mut op = DeltaOperation::insert("HellWorld");
        insert_at(&mut op, 4, "o ")?;
        assert_eq!(op.insert_value().str_val()?, "Hello World".to_string());
        Ok(())
    }

    #[test]
    fn delete_at_test() -> Result<()> {
        let mut op = DeltaOperation::insert("Hello World");
        delete_at(&mut op, 5, 1)?;
        assert_eq!(op.insert_value().str_val()?, "HelloWorld".to_string());

        let mut op = DeltaOperation::insert("Hello World");
        delete_at(&mut op, 6, 5)?;
        assert_eq!(op.insert_value().str_val()?, "Hello ".to_string());

        let mut op = DeltaOperation::insert("Hello World");
        delete_at(&mut op, 0, 5)?;
        assert_eq!(op.insert_value().str_val()?, " World".to_string());

        let mut op = DeltaOperation::insert("Hello World");
        delete_at(&mut op, 0, 11)?;
        assert_eq!(op.insert_value().str_val()?, "".to_string());
        Ok(())
    }

    #[test]
    fn merge_test() -> Result<()> {
        let left = DeltaOperation::insert("Hello ");
        let right = DeltaOperation::insert("World");
        let op = merge(&left, &right)?;
        assert_eq!(op.insert_value().str_val()?, "Hello World".to_string());
        Ok(())
    }

    #[test]
    fn split_at_test() -> Result<()> {
        let op = DeltaOperation::insert("Hello World");
        let (left, right) = split_at(&op, 5)?;
        assert_eq!(left.insert_value().str_val()?, "Hello".to_string());
        assert_eq!(right.insert_value().str_val()?, " World".to_string());

        let op = DeltaOperation::insert("Hello World");
        let (left, right) = split_at(&op, 0)?;
        assert_eq!(left.insert_value().str_val()?, "".to_string());
        assert_eq!(right.insert_value().str_val()?, "Hello World".to_string());

        let op = DeltaOperation::insert("Hello World");
        let (left, right) = split_at(&op, 11)?;
        assert_eq!(left.insert_value().str_val()?, "Hello World".to_string());
        assert_eq!(right.insert_value().str_val()?, "".to_string());
        Ok(())
    }
}
