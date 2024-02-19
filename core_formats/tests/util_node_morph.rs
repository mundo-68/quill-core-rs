use core_formats::paragraph::Pblock;
use core_formats::text_formatter::TextFormat;
use core_formats::util::node_morph::{
    delete_node, insert_empty_block_node_after_cursor, merge_block_node, merge_text_node,
    split_block_at_cursor, split_block_before_child, split_text_and_block_at_cursor,
    split_text_node, try_3_way_merge_text,
};
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use node_tree::cursor::{Cursor, CursorLocation};
use node_tree::dom_doc_tree_morph::append;
use node_tree::format_trait::FormatTait;
use node_tree::tree_traverse::next_sibling;
use op_transform::doc_root::DocumentRoot;
use std::sync::Arc;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// The util_node_morph module does not add `<br/>`. So here we do not add it to the tests

fn create_text(doc: &DocumentRoot) -> anyhow::Result<()> {
    let p_format = Arc::new(Pblock::new());
    let t_format = Arc::new(TextFormat::new());

    let mut attr = Attributes::default();
    attr.insert("bold", true);

    let root = doc.get_root();

    let delta = DeltaOperation::insert("\n");
    let par = p_format.create(delta, p_format.clone())?;
    append(root, par.clone());

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
    //let sb = DomElement::new("br");
    //par.get_dom_element().unwrap().insert_child(0, sb.node());
    append(&root, par.clone());
    Ok(())
}

#[wasm_bindgen_test]
fn create_text_test() -> anyhow::Result<()> {
    let doc = DocumentRoot::new("create_text_test");
    doc.append_to_body();
    create_text(&doc)?;

    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

/// From delete_node() ...
/// -  If we are a block, then deleting the block should only happen for an empty block.
#[wasm_bindgen_test]
fn delete_node_test() -> anyhow::Result<()> {
    let doc = DocumentRoot::new("delete_node_test");
    doc.append_to_body();
    create_text(&doc)?;

    let root = doc.get_root();
    let children = root.get_children();
    let par = children.get(0).unwrap();
    let children = par.get_children();
    let text = children.get(1).unwrap();
    delete_node(text);

    let expect = r#"<p>TEXT_1_1TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let children = par.get_children();
    let text = children.get(1).unwrap();
    delete_node(text);

    let expect = r#"<p>TEXT_1_1</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let children = par.get_children();
    let text = children.get(0).unwrap();
    delete_node(text);

    let expect = r#"<p></p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    delete_node(par);

    let expect = r#"<p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
pub fn merge_block_node_test() -> anyhow::Result<()> {
    let doc = DocumentRoot::new("merge_block_node_test");
    doc.append_to_body();
    create_text(&doc)?;

    let root = doc.get_root();
    let children = root.get_children();
    let left = children.get(0).unwrap();
    let right = children.get(1).unwrap();

    merge_block_node(&left, &right)?;
    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
pub fn split_block_before_child_test() -> anyhow::Result<()> {
    let doc = DocumentRoot::new("merge_block_node_test");
    doc.append_to_body();
    create_text(&doc)?;

    let root = doc.get_root();
    let children = root.get_children();
    let par = children.get(0).unwrap();
    let children = par.get_children();
    let block = children.get(2).unwrap();

    split_block_before_child(&par, block)?;
    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong></p><p>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
pub fn merge_text_node_test() -> anyhow::Result<()> {
    let doc = DocumentRoot::new("merge_block_node_test");
    doc.append_to_body();
    create_text(&doc)?;

    let root = doc.get_root();
    let children = root.get_children();
    let par = children.get(0).unwrap();
    let children = par.get_children();
    let left = children.get(0).unwrap();
    let middle = children.get(1).unwrap();
    let right = children.get(2).unwrap();

    delete_node(middle); //so we can merge delta with same properties

    merge_text_node(&left, &right)?;
    let expect = r#"<p>TEXT_1_1TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
pub fn split_text_node_test() -> anyhow::Result<()> {
    let doc = DocumentRoot::new("split_text_node_test");
    doc.append_to_body();
    create_text(&doc)?;

    let root = doc.get_root();
    let children = root.get_children();
    let par = children.get(0).unwrap();
    let children = par.get_children();
    let left = children.get(0).unwrap();
    let middle = children.get(1).unwrap();

    split_text_node(&middle, 4)?;
    let expect = r#"<p>TEXT_1_1<strong>TEXT</strong><strong>_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    split_text_node(&left, 4)?;
    let count = par.get_children().len();
    assert_eq!(count, 5);
    //Text does not show split TEXT_1_1, but the child count above does ...
    let expect = r#"<p>TEXT_1_1<strong>TEXT</strong><strong>_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
pub fn insert_empty_block_node_after_cursor_test() -> anyhow::Result<()> {
    let doc = DocumentRoot::new("insert_empty_block_node_after_cursor_test");
    doc.append_to_body();
    create_text(&doc)?;

    let root = doc.get_root();
    let children = root.get_children();
    let last_par = children.get(2).unwrap();
    let first_par = children.get(0).unwrap();
    let children = first_par.get_children();
    let last = children.get(2).unwrap();

    let cursor = Cursor::new();
    cursor.set_after(last);
    insert_empty_block_node_after_cursor(&cursor)?;

    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p></p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    cursor.set_at(last_par, 0);
    insert_empty_block_node_after_cursor(&cursor)?;

    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p></p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
pub fn split_block_at_cursor_test() -> anyhow::Result<()> {
    let doc = DocumentRoot::new("split_block_at_cursor_test");
    doc.append_to_body();
    create_text(&doc)?;

    let root = doc.get_root();
    let children = root.get_children();
    let last_par = children.get(2).unwrap();
    let first_par = children.get(0).unwrap();
    let children = first_par.get_children();
    let first = children.get(0).unwrap();
    let last = children.get(2).unwrap();

    let cursor = Cursor::new();

    cursor.set_before(last);
    split_block_at_cursor(&cursor)?;
    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong></p><p>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    cursor.set_after(first);
    split_block_at_cursor(&cursor)?;
    let expect = r#"<p>TEXT_1_1</p><p><strong>TEXT_1_2</strong></p><p>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    cursor.set_at(last_par, 0);
    insert_empty_block_node_after_cursor(&cursor)?;

    let expect = r#"<p>TEXT_1_1</p><p><strong>TEXT_1_2</strong></p><p>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
pub fn split_text_and_block_at_cursor_test() -> anyhow::Result<()> {
    let doc = DocumentRoot::new("split_text_and_block_at_cursor_test");
    doc.append_to_body();
    create_text(&doc)?;

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
    cursor.set_at(&middle, 4);
    split_text_and_block_at_cursor(&cursor)?;
    let expect = r#"<p>TEXT_1_1<strong>TEXT</strong></p><p><strong>_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //error!( "You are at tests 2");
    cursor.set_at(&left, 4);
    split_text_and_block_at_cursor(&cursor)?;
    let expect = r#"<p>TEXT</p><p>_1_1<strong>TEXT</strong></p><p><strong>_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    // error!( "You are at tests 3");
    cursor.set_after(&right);
    split_text_and_block_at_cursor(&cursor)?;
    let expect = r#"<p>TEXT</p><p>_1_1<strong>TEXT</strong></p><p><strong>_1_2</strong>TEXT_1_3</p><p></p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //error!( "You are at tests 4");
    cursor.set_at(&last_par, 0);
    split_text_and_block_at_cursor(&cursor)?;
    let expect = r#"<p>TEXT</p><p>_1_1<strong>TEXT</strong></p><p><strong>_1_2</strong>TEXT_1_3</p><p></p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
pub fn try_3_way_merge_text_test() -> anyhow::Result<()> {
    //----------------------------------------------------------------
    // Splitting regular text
    //----------------------------------------------------------------
    let doc = DocumentRoot::new("try_3_way_merge_text_test_A");
    doc.append_to_body();
    create_text(&doc)?;

    //----------------------------------------------------------------
    // Check created document
    //error!("------- tests 1 --------" );
    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let root = doc.get_root();
    let children = root.get_children();
    let par = children.get(0).unwrap();
    let children = par.get_children();
    let left = children.get(0).unwrap(); //TEXT_1_1
    assert_eq!(par.child_count(), 3);

    //----------------------------------------------------------------
    //error!("------- tests 2 --------" );
    // split <p>[TEXT][_1_1]<strong>TEXT_1_2</strong>TEXT_1_3</p>...
    split_text_node(&left, 4)?;
    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(par.child_count(), 4);
    assert_eq!(doc.as_html_string(), expect);
    assert_eq!(left.get_operation().insert_value().str_val()?, "TEXT");

    //----------------------------------------------------------------
    //error!("------- tests 3 --------" );
    // split <p>[TE][XT][_1_1]<strong>TEXT_1_2</strong>TEXT_1_3</p>...
    split_text_node(&left, 2)?;
    assert_eq!(par.child_count(), 5);
    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //----------------------------------------------------------------
    // merge by setting cursor <p>[TE][XT][{*}_1_1]<strong>TEXT_1_2</strong>TEXT_1_3</p>...
    //error!("------- tests 4 --------" );
    let merge_me = next_sibling(&left).unwrap();
    assert_eq!(merge_me.get_operation().insert_value().str_val()?, "XT");

    // NOTE: Normally the cursor points one beyond the previously inserted / retained document node
    let cursor = Cursor::new();
    cursor.set_before(&next_sibling(&merge_me).unwrap());
    assert_eq!(
        cursor
            .get_doc_node()
            .get_operation()
            .insert_value()
            .str_val()?,
        "_1_1"
    );

    try_3_way_merge_text(&cursor)?;

    assert_eq!(par.get_children().len(), 3);
    assert_eq!(
        cursor.get_doc_node().get_dom_text().unwrap().get_text(),
        "TEXT_1_1"
    );
    match cursor.get_location() {
        CursorLocation::At(_dn, i) => {
            assert_eq!(i, 4);
        }
        _ => {
            assert!(false); // should be at
        }
    }
    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //----------------------------------------------------------------
    // Splitting bold format
    //----------------------------------------------------------------
    //error!("------- tests 5 --------" );
    let doc = DocumentRoot::new("try_3_way_merge_text_test_A");
    doc.append_to_body();
    create_text(&doc)?;

    let root = doc.get_root();
    let children = root.get_children();
    let par = children.get(0).unwrap();
    let children = par.get_children();
    let middle = children.get(1).unwrap();

    //split <p>TEXT_1_1<strong>[TEXT][_1_2]</strong>TEXT_1_3</p>
    split_text_node(&middle, 4)?;
    let expect = r#"<p>TEXT_1_1<strong>TEXT</strong><strong>_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //split <p>TEXT_1_1<strong>[TE][XT][_1_2]</strong>TEXT_1_3</p>
    split_text_node(&middle, 2)?;
    let expect = r#"<p>TEXT_1_1<strong>TE</strong><strong>XT</strong><strong>_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let merge_me = next_sibling(&middle).unwrap();
    assert_eq!(merge_me.get_operation().insert_value().str_val()?, "XT");

    let cursor = Cursor::new();
    cursor.set_before(&next_sibling(&merge_me).unwrap());
    try_3_way_merge_text(&cursor)?;
    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}
