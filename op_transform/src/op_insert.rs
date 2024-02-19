// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::auto_soft_break::AutomaticSoftBreak;
use crate::error::Error::UnexpectedCursorPosition;
use crate::registry::Registry;
use anyhow::Result;
use core_formats::util::node_morph::split_text_and_block_at_cursor;
use delta::operations::DeltaOperation;
use node_tree::cursor::{Cursor, CursorLocation};
use node_tree::dom_doc_tree_morph::{append, insert_after, insert_before};
use node_tree::format_trait::FormatTait;
use node_tree::tree_traverse::{next_sibling, prev_node_non_zero_length};
use std::sync::{Arc, RwLockReadGuard};

/// # insert()
///
/// Insert() starts at the given cursor position.
/// To handle more complex formats, we should isolate the current cursor position.
///
/// Remember: Insert happens `[BEFORE]` the cursor, leaving the cursor in the next
/// insert position.
/// ```bash
///   [node 1]{*}[node 2][node 3]
/// ```
/// Now we `isolate()` the node that may change from the insert operation, at `node 1`. If it is an
/// insertion of text, then we add `new content` or if there is an insertion of a block node
/// we may even change the block node to which `node 1` belongs ....
///
/// ```bash
///      [node 0][[node 1]]{*}[node 2][node 3]
///         --> (insert)
///      [node 0][[node 1]][new content]{*}[node 2][node 3]
///         ^         ^          ^             ^
///         |         |          |             |
///  Boundaries:  A         B           C
/// ```
/// And the `merge()` operation should try to merge the nodes depicted at the arrows (pointing up).
/// Notice that these are 3! boundaries that possibly need repair. If we start using the format
/// of `[new content]` then we should repair:
///  - `boundary A`: Check if `[node 0]` and `[node 1]` still match using format `1`<br>
///     Normally `[node 0]` and `[node 1]` are not split. so we can ignore this case. But for formats which do split theres ...merge them too!
///  - `boundary B`: Check if `[node 1]` and `[new content]` match using format `[new content]`
///  - `boundary C`: Check if `[new content]` and `[node 2]` match using format `[new content]`
/// This is all lumped together in a function `try_merge()`
///
/// FIXME: Depending on cursor position we get a different cursor location AFTER `insert(\n)`
/// FIXME: either end of previous line OR start of next line
/// FIXME: Must be start of next line !!

/// `Insert()` inserts the character BEFORE the current cursor position.
/// as a consequence, the cursor position should not need adaptation after insert.
///
/// When a block node is inserted, it should only change the bock format, and not
/// contain any text. This is done in the input by splitting all `DeltaOperation`
/// text in elements that are either `Insert('some text')` or `Insert('\n')`
///
/// Retain index: The retain index changes by the length of the delta operation on the input.
pub fn insert(
    cursor: &Cursor,
    op: DeltaOperation,
    reg: &RwLockReadGuard<'static, Registry>,
) -> Result<()> {
    let retain = cursor.get_retain_index();
    cursor.set_retain_index(retain + op.op_len());

    if reg.is_block_fmt(&op)? {
        insert_new_block(cursor, op, &reg)?;
    } else {
        let new_format = reg.line_format(&op)?;
        insert_new_text(cursor, op, &new_format)?;
    }

    //We should not stick the cursor to a DOC node with length 0
    assert!(
        cursor.get_doc_node().get_formatter().is_text_format()
            && cursor.get_doc_node().op_len() > 0
            || !cursor.get_doc_node().get_formatter().is_text_format()
    );
    Ok(())
}

//---------------------------------------------------------------------
// Helper functions
//---------------------------------------------------------------------

/// # insert_new_block()
///
/// Inserts a block in some `DocumentNode`.
///
/// Example, assuming we insert a header block `<H1>` at the cursor <br>
///  `<p>header{*}and text</p>`
/// results in <br>
/// `<H1>header<H1><p>{*}and text</p>`
///
/// Example, inserting a header in an empty P-block:<br>
///  `<p>{*}</p>`<br>
/// results in <br>
/// `<H1><H1><p>{*}</p>`
///
/// Where `{*}` denotes the cursor location.
fn insert_new_block(
    cursor: &Cursor,
    delta: DeltaOperation,
    registry: &RwLockReadGuard<'static, Registry>,
) -> Result<()> {
    if cursor.get_doc_node().get_formatter().is_text_format() {
        //See explanation above this module on splitting of text before inserting
        split_text_and_block_at_cursor(cursor)?;

        let left_node = prev_node_non_zero_length(&cursor.get_doc_node()).unwrap();
        let left_parent = if left_node.get_formatter().is_text_format() {
            left_node.get_parent().unwrap()
        } else {
            //left_node
            left_node.clone()
        };

        //Remember if we split or not ...
        let (right_parent, right_format) = if let Some(right_parent) = next_sibling(&left_parent) {
            let format = right_parent.get_formatter();
            format.isolate(&right_parent)?;
            if right_parent.is_empty_block() {
                AutomaticSoftBreak::insert(&right_parent)?;
            }
            (Some(right_parent), Some(format))
        } else {
            (None, None)
        };

        let left_format = left_parent.get_formatter();
        let left_parent = left_format.un_block_transform(cursor, &left_parent)?;

        let new_format = registry.block_format(&delta)?;
        let left_parent =
            new_format.block_transform(cursor, &left_parent, delta, new_format.clone())?;
        if left_parent.is_empty_block() {
            AutomaticSoftBreak::insert(&left_parent)?;
        }

        //Now merge again but only if we split before ...
        //try_merge() also merges 2 nodes to the left ...
        if let Some(right_parent) = right_parent {
            right_format.unwrap().try_merge(&cursor, &right_parent)?;
        }
    } else {
        //inserting block in/after an (empty) block.
        //The new block is empty so it gets a line break

        //Isolate right hand node. This is right hand, because the cursor is "right" before
        //the new block command. Inserted text would go IN-to the block, so a new block,
        //goes BEFORE the block
        let right = cursor.get_doc_node();
        let right_format = right.get_formatter();
        let right = right_format.isolate(&right)?; //isolated right ...

        //create new formatted node
        let left_format = registry.block_format(&delta)?;
        let left = left_format.create(delta, left_format.clone())?;

        //in case we are an zero length block like UL block ...
        if left.op_len() > 0 {
            AutomaticSoftBreak::insert(&left)?;
        } else {
            AutomaticSoftBreak::insert(&prev_node_non_zero_length(&left).unwrap())?;
        }

        //Insert the new node in the right spot
        insert_before(&right.get_parent().unwrap(), &right, left.clone());

        //Merge all again ...
        left_format.try_merge(cursor, &left)?;
        //Maybe we already resolve this, so we tests if right still exists in the tree
        if left.get_parent().unwrap().get_children().contains(&right) {
            right_format.try_merge(cursor, &right)?;
        }

        //NO cursor update, because we are still at the right hand block
    }
    Ok(())
}

/// # insert_new_text()
///
/// Inserts a text in some `DocumentNode`.
///
/// Splitting the text nodes, and merging them again, takes care of the possible
/// formatting differences between the new to be inserted `DeltaOperation`, and the
/// formatting of the `DeltaOperation` pointed to by the cursor.
///
/// Note:<br>
/// The agreed post condition of `FormatTrait::split_leaf()` is to leave the cursor `BEFORE` the
/// right hand node.
///
//FIXME: Do not set cursor to doc node with length = 0 --> find proper position
//FIXME: next / prev should skip doc node with length = 0 ...?
fn insert_new_text(
    cursor: &Cursor,
    delta: DeltaOperation,
    format: &Arc<dyn FormatTait + Sync + Send>,
) -> Result<()> {
    // Take care of the cursor currently pointing AT[block, 0] in a block node
    // By definition this block parent node is empty, and should be populated with this new leaf
    if let CursorLocation::At(block, _idx) = cursor.get_location() {
        if !block.get_formatter().is_text_format() && block.child_count() == 0 {
            //empty blocks have soft brake, but this one is not empty anymore ...
            AutomaticSoftBreak::remove(&block)?;

            let doc_node = format.create(delta, format.clone())?;
            cursor.set_after_no_retain_update(&doc_node);
            append(&block, doc_node.clone());
            //there should be nothing to merge but ...
            //try_merge() is counterpart of isolate() so we may need to move the cursor to
            //a document node which has op_len() <> 0 ...
            format.try_merge(&cursor, &doc_node)?;
            return Ok(());
        }
    }

    //See explanation above this module on splitting of text before inserting
    cursor.get_doc_node().get_formatter().split_leaf(cursor)?;

    match cursor.get_location() {
        CursorLocation::After(doc_node) => {
            let new_node = format.create(delta, format.clone())?;
            insert_after(&doc_node.get_parent().unwrap(), &doc_node, &new_node);
            cursor.set_after_no_retain_update(&new_node);
        }
        CursorLocation::Before(doc_node) => {
            let new_node = format.create(delta, format.clone())?;
            cursor.set_after_no_retain_update(&new_node);
            insert_before(&doc_node.get_parent().unwrap(), &doc_node, new_node.clone());
        }
        CursorLocation::At(_doc_node, _index) => {
            //cursor.get_doc_node().get_formatter().split_leaf(cursor);
            let right = cursor.get_doc_node();
            let new_node = format.create(delta, format.clone())?;
            insert_before(&right.get_parent().unwrap(), &right, new_node.clone());
            cursor.set_after_no_retain_update(&new_node);
        }
        CursorLocation::None => {
            return Err(UnexpectedCursorPosition {
                pos: "None".to_string(),
            }
            .into());
        }
    };

    format.try_merge(&cursor, &cursor.get_doc_node())?;
    Ok(())
}

//---------------------------------------------------------------------
// Test code
//---------------------------------------------------------------------
#[cfg(test)]
mod test {
    use crate::doc_root::DocumentRoot;
    use crate::op_insert;
    use crate::registry::{init_test_registry, Registry};
    use anyhow::Result;
    use core_formats::text_formatter::TextFormat;
    use core_formats::{P_FORMAT, TEXT_FORMAT};
    use delta::attributes::Attributes;
    use delta::operations::DeltaOperation;
    use dom::dom_element::DomElement;
    use node_tree::cursor::Cursor;
    use node_tree::dom_doc_tree_morph::append;
    use node_tree::format_trait::FormatTait;
    use std::ops::Deref;
    use std::sync::Arc;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    pub fn insert_text_test() -> Result<()> {
        let doc = DocumentRoot::new("insert_text_test");
        doc.append_to_body();
        create_text(&doc)?;

        //let p_format: Arc<dyn FormatTait+ Send + Sync> = Arc::new(Pblock::new());
        let t_format: Arc<dyn FormatTait + Send + Sync> = Arc::new(TextFormat::new());

        let root = doc.get_root();
        let children = root.get_children();
        let last_par = children.get(2).unwrap();
        let first_par = children.get(0).unwrap();
        let children = first_par.get_children();
        let left = children.get(0).unwrap();
        let middle = children.get(1).unwrap();
        let right = children.get(2).unwrap();

        let cursor = Cursor::new();

        //error!( "You are at tests 1");
        let delta = DeltaOperation::insert("hello");
        cursor.set_at(&left, 4);
        op_insert::insert_new_text(&cursor, delta, &t_format)?;
        let expect = r#"<p>TEXThello_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
        assert_eq!(doc.as_html_string(), expect);

        //error!( "You are at tests 2");
        cursor.set_at(&middle, 4);
        let delta = DeltaOperation::insert("hello");
        op_insert::insert_new_text(&cursor, delta, &t_format)?;
        let expect = r#"<p>TEXThello_1_1<strong>TEXT</strong>hello<strong>_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
        assert_eq!(doc.as_html_string(), expect);

        //error!( "You are at tests 3");
        cursor.set_at(&last_par, 0);
        let delta = DeltaOperation::insert("sweet");
        op_insert::insert_new_text(&cursor, delta, &t_format)?;
        let expect = r#"<p>TEXThello_1_1<strong>TEXT</strong>hello<strong>_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p>sweet</p>"#;
        assert_eq!(doc.as_html_string(), expect);

        //error!( "You are at tests 4");
        let mut attr = Attributes::default();
        attr.insert("italic", true);
        cursor.set_before(&right);
        let delta = DeltaOperation::insert_attr("myhome", attr);
        op_insert::insert_new_text(&cursor, delta, &t_format)?;
        let expect = r#"<p>TEXThello_1_1<strong>TEXT</strong>hello<strong>_1_2</strong><em>myhome</em>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p>sweet</p>"#;
        assert_eq!(doc.as_html_string(), expect);
        Ok(())
    }

    #[wasm_bindgen_test]
    pub fn insert_block_test() -> Result<()> {
        init_test_registry();
        let registry = Registry::get_ref()?;

        let doc = DocumentRoot::new("insert_block_test");
        doc.append_to_body();
        create_text(&doc)?;

        let cursor = Cursor::new();

        let root = doc.get_root();
        let children = root.get_children();
        let last_par = children.get(2).unwrap();
        let first_par = children.get(0).unwrap();
        let children = first_par.get_children();
        let left = children.get(0).unwrap();

        //error!( "You are at tests 1");
        //adding empty block
        cursor.set_at(&last_par, 0);
        let delta = DeltaOperation::insert("\n");
        op_insert::insert_new_block(&cursor, delta, &registry)?;

        let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p><p><br></p>"#;
        assert_eq!(doc.as_html_string(), expect);

        //error!( "You are at tests 2");
        //checking cursor by adding text at the cursor
        let delta = DeltaOperation::insert("you are here");
        op_insert::insert_new_text(&cursor, delta, &TEXT_FORMAT.deref().clone())?;
        let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p><p>you are here</p>"#;
        assert_eq!(doc.as_html_string(), expect);

        //error!( "You are at tests 3");
        //splitting first text block
        cursor.set_at(&left, 4);
        //error!( "cursor tests 3 {}", &cursor);
        let delta = DeltaOperation::insert("\n");
        op_insert::insert_new_block(&cursor, delta, &registry)?;
        let expect = r#"<p>TEXT</p><p>_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p><p>you are here</p>"#;
        assert_eq!(doc.as_html_string(), expect);

        //error!( "You are at tests 4");
        //Checking cursor by adding text
        let delta = DeltaOperation::insert("here");
        op_insert::insert_new_text(&cursor, delta, &TEXT_FORMAT.deref().clone())?;

        let expect = r#"<p>TEXT</p><p>here_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p><p>you are here</p>"#;
        assert_eq!(doc.as_html_string(), expect);
        Ok(())
    }

    fn create_text(doc: &DocumentRoot) -> Result<()> {
        let p_format = &P_FORMAT.deref().clone();
        let t_format = &TEXT_FORMAT.deref().clone();

        let mut attr = Attributes::default();
        attr.insert("bold", true);

        let root = doc.get_root();

        let delta = DeltaOperation::insert("\n");
        let par = p_format.create(delta, p_format.clone())?;
        append(&root, par.clone());

        let delta = DeltaOperation::insert("TEXT_1_1");
        let t = t_format.create(delta, t_format.clone())?;
        append(&par, t.clone());

        let delta = DeltaOperation::insert_attr("TEXT_1_2", attr.clone());
        let t = t_format.create(delta, t_format.clone())?;
        append(&par, t.clone());

        let delta = DeltaOperation::insert("TEXT_1_3");
        let t = t_format.create(delta, t_format.clone())?;
        append(&par, t.clone());

        let delta = DeltaOperation::insert("\n");
        let par = p_format.create(delta, p_format.clone())?;
        append(&root, par.clone());

        let delta = DeltaOperation::insert("TEXT_2_1");
        let t = t_format.create(delta, t_format.clone())?;
        append(&par, t.clone());

        let delta = DeltaOperation::insert_attr("TEXT_2_2", attr.clone());
        let t = t_format.create(delta, t_format.clone())?;
        append(&par, t.clone());

        let delta = DeltaOperation::insert("\n");
        let par = p_format.create(delta, p_format.clone())?;
        let sb = DomElement::new("br");
        par.get_dom_element().unwrap().insert_child(0, sb.node());
        append(&root, par.clone());
        Ok(())
    }

    #[wasm_bindgen_test]
    fn create_text_test() -> Result<()> {
        let doc = DocumentRoot::new("create_text_test");
        doc.append_to_body();
        create_text(&doc)?;

        let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
        assert_eq!(doc.as_html_string(), expect);
        Ok(())
    }
}
