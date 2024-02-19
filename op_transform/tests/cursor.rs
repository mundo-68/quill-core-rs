#[path = "op_transform_test_utils.rs"]
mod op_transform_test_utils;

use anyhow::Result;
use delta::attributes::Attributes;
use delta::delta::Delta;
use delta::document::Document;
use delta::operations::DeltaOperation;
use node_tree::cursor::{Cursor, CursorLocation};
use node_tree::doc_node::DocumentNode;
use node_tree::tree_traverse::{first_node, last_block_node, last_leaf_node, next_node, prev_node};
use op_transform::doc_root::DocumentRoot;
use op_transform::registry::init_test_registry;
use std::sync::Arc;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn retain_index_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("retain_index_test");
    let mut doc2 = doc.clone();
    doc.append_to_body();

    //-----------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("0123456789");
    doc.close();
    doc.open()?;
    doc.apply_delta(delta)?;

    let dn = last_block_node(doc.get_root());
    let prev = prev_node(&dn.unwrap()).unwrap();
    let cursor = doc.get_cursor();
    cursor.set_after(&prev);
    assert_eq!(cursor.get_retain_index(), 10);

    //-----------------------------------------------------------------------
    cursor.set_before(&first_node(doc.get_root()));
    assert_eq!(cursor.get_retain_index(), 0);
    doc.cursor_to_start();
    assert_eq!(cursor.get_retain_index(), 0);

    //-----------------------------------------------------------------------
    cursor.set_at(&first_node(doc.get_root()), 3);
    assert_eq!(cursor.calculate_retain_index(), 3);
    assert_eq!(cursor.get_retain_index(), 3);
    doc.cursor_to_start();
    assert_eq!(cursor.get_retain_index(), 0);

    //-----------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("0123456789");
    delta.insert("\n");
    delta.insert("abcdefghij");

    doc2.open()?;
    doc2.apply_delta(delta)?;
    let cursor = doc.get_cursor();

    let dn = last_block_node(doc.get_root()).unwrap();
    let prev = prev_node(&dn).unwrap();
    cursor.set_after(&prev);
    assert_eq!(cursor.get_retain_index(), 21);

    //-----------------------------------------------------------------------
    doc.cursor_to_end();
    assert_eq!(cursor.get_retain_index(), 21);

    //-----------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(16);
    doc.reset_cursor();
    doc.apply_delta(delta)?;
    let cur = doc.get_cursor();
    assert_eq!(cur.get_retain_index(), 16);
    Ok(())
}

#[wasm_bindgen_test]
fn selection_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("backspace_test");
    doc.append_to_body();

    let mut attr1 = Attributes::default();
    attr1.insert("bold", true);

    let attr2 = Attributes::default();
    attr1.insert("italic", true);

    let mut delta = Delta::default();
    delta.insert_attr("0123", attr1);
    delta.insert("456");
    delta.insert_attr("789", attr2);
    delta.insert("abcd");

    doc.open()?;
    doc.apply_delta(delta)?;

    //---------------------------------------------------
    doc.reset_cursor();
    let mut delta = Delta::default();
    delta.retain(1);
    doc.apply_delta(delta)?;
    let start = doc.get_cursor().clone();

    let mut delta = Delta::default();
    delta.retain(6);
    doc.apply_delta(delta)?;
    let end = doc.get_cursor().clone();

    let cursor = doc.get_cursor();
    cursor.set_select_start(start.get_select_start());
    cursor.set_select_stop(end.get_select_start());

    assert_eq!(cursor.get_retain_index(), 1);
    assert_eq!(cursor.selection_length(), 5);
    Ok(())
}

#[wasm_bindgen_test]
fn backspace_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("backspace_test");
    doc.append_to_body();
    let mut doc2 = doc.clone();
    let cursor = Cursor::new();

    //-----------------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("bold", true);

    let mut delta = Delta::default();
    delta.insert("abcdefghij");
    delta.insert_attr("zyxwv\nusrqp", attr);
    delta.insert("\n");
    delta.insert("0123456789");
    doc.close();
    doc2.open()?;
    doc2.apply_delta(delta)?;
    //Check string of what is expected to be pointed at
    //Adding '@' as marker for <P> blocks
    let txt = "abcdefghijzyxwv@usrqp@0123456789@";

    let dn = last_leaf_node(doc2.get_root()).unwrap();
    cursor.set_after(&dn);

    assert_eq!(cursor.get_retain_index(), 32 as usize);
    assert_eq!(Delta::document_length(&doc.to_delta()), 33 as usize); //includes last <P>

    let t: usize = 32;
    for i in 0..32 as usize {
        assert!(cursor.get_doc_node().is_leaf());
        assert_eq!(cursor.get_retain_index(), t - i);

        let c = txt.chars().nth(t - i).unwrap();
        if c.to_string() == "@".to_string() {
            assert_eq!(cursor_points_to(&cursor), "<P>");
        } else {
            assert_eq!(cursor_points_to(&cursor), c.to_string())
        }
        cursor.backspace()?;
    }
    Ok(())
}

#[wasm_bindgen_test]
fn cursor_util_char_pointed_to_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("cursor_util_char_pointed_to_test");
    doc.append_to_body();

    //-----------------------------------------------------------------------
    let str = "abcdefghij@0123456789@".to_string();

    let mut delta = Delta::default();
    delta.insert("abcdefghij");
    delta.insert("\n");
    delta.insert("0123456789");

    doc.open()?;
    doc.apply_delta(delta)?;

    for i in 0..Delta::document_length(&doc.to_delta()) {
        doc.reset_cursor();
        let c = str.chars().nth(i).unwrap();
        doc.apply_operation(DeltaOperation::retain(i))?;
        if c.to_string() == "@".to_string() {
            assert_eq!(cursor_points_to(doc.get_cursor()), "<P>");
        } else {
            assert_eq!(cursor_points_to(doc.get_cursor()), c.to_string())
        }
    }
    Ok(())
}

/// Returns the text, or block-type to which the cursor points.
pub fn cursor_points_to(cursor: &Cursor) -> String {
    match cursor.get_location() {
        CursorLocation::None => {
            panic!("hey no location")
        }
        CursorLocation::After(doc_node) => {
            if let Some(next) = next_node(&doc_node) {
                doc_node_to_str(&next, 0)
            } else {
                " -- beyond last node -- ".to_string()
            }
        }
        CursorLocation::Before(doc_node) => doc_node_to_str(&doc_node, 0),
        CursorLocation::At(doc_node, index) => doc_node_to_str(&doc_node, index),
    }
}

fn doc_node_to_str(doc_node: &Arc<DocumentNode>, index: usize) -> String {
    if doc_node.is_text() {
        if doc_node.op_len() <= index {
            let txt = doc_node.get_doc_dom_node().get_node_name();
            return txt;
        }
        let txt = doc_node.find_dom_text().get_text();
        txt[index..index + 1].to_string()
    } else {
        if index != 0 {
            assert_eq!(index, 0);
        }
        let txt = doc_node.get_doc_dom_node().get_node_name();
        format!("<{}>", txt)
    }
}
