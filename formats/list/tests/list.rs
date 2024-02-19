use anyhow::Result;
use core_formats::format_const::{NAME_P_BLOCK, NAME_TEXT};
use core_formats::{P_FORMAT, TEXT_FORMAT};
use delta::attributes::Attributes;
use delta::delta::Delta;
use delta::types::attr_val::AttrVal;
use list::list_const::{LIST_ATTR_KEY, LIST_BULLET};
use list::{ListBlock, NAME_OL_BLOCK, NAME_UL_BLOCK};
use op_transform::doc_root::DocumentRoot;
use op_transform::registry::Registry;
use std::ops::Deref;
use std::sync::{Arc, Mutex, OnceLock};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// The test registry registers only the BASIC formats required for testing in this module
static TEST_REGISTRY: OnceLock<Mutex<usize>> = OnceLock::new();
fn init_test_registry() {
    TEST_REGISTRY.get_or_init(|| {
        Registry::init_registry();
        let mut r = Registry::get_mut_ref().unwrap();
        r.register_block_fmt(NAME_UL_BLOCK, Arc::new(ListBlock::new_ul()))
            .unwrap();
        r.register_block_fmt(NAME_OL_BLOCK, Arc::new(ListBlock::new_ol()))
            .unwrap();
        r.register_block_fmt(NAME_P_BLOCK, P_FORMAT.deref().clone())
            .unwrap();
        r.register_line_fmt(NAME_TEXT, TEXT_FORMAT.deref().clone())
            .unwrap();
        Mutex::new(1)
    });
}

#[wasm_bindgen_test]
fn list_create_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("list_create_test");
    doc.append_to_body();

    let mut attr = Attributes::default();
    attr.insert(LIST_ATTR_KEY, LIST_BULLET);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("Leading text\nfirst");
    delta.insert_attr("\n", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading text</p><ul><li>first</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(19);
    delta.insert("second");
    delta.insert_attr("\n", attr.clone());

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading text</p><ul><li>first</li><li>second</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(26);
    delta.insert("third");
    delta.insert_attr("\n", attr.clone());

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect =
        r#"<p>Leading text</p><ul><li>first</li><li>second</li><li>third</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(31);
    delta.insert_attr("\n", attr.clone());

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading text</p><ul><li>first</li><li>second</li><li>third</li><li><br></li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn list_insert_text_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("list_insert_text_test");
    doc.append_to_body();

    let mut attr = Attributes::default();
    attr.insert(LIST_ATTR_KEY, LIST_BULLET);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("Leading\nfirst");
    delta.insert_attr("\n", attr.clone());
    delta.insert("second");
    delta.insert_attr("\n", attr.clone());
    delta.insert("third");
    delta.insert_attr("\n", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading</p><ul><li>first</li><li>second</li><li>third</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(8);
    delta.insert("QQ");

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect =
        r#"<p>Leading</p><ul><li>QQfirst</li><li>second</li><li>third</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(22);
    delta.insert("QQ");

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect =
        r#"<p>Leading</p><ul><li>QQfirst</li><li>secondQQ</li><li>third</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(30);
    delta.insert("QQ");

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect =
        r#"<p>Leading</p><ul><li>QQfirst</li><li>secondQQ</li><li>thirdQQ</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(33);
    delta.insert("QQ");

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect =
        r#"<p>Leading</p><ul><li>QQfirst</li><li>secondQQ</li><li>thirdQQ</li></ul><p>QQ</p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn list_delete_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("list_insert_text_test");
    doc.append_to_body();

    let mut attr = Attributes::default();
    attr.insert(LIST_ATTR_KEY, LIST_BULLET);

    //--------------------------------------------------------------------------
    //error!("list_delete_test() tests 1");
    let mut delta = Delta::default();
    delta.insert("Leading\nfirst");
    delta.insert_attr("\n", attr.clone());
    delta.insert("second");
    delta.insert_attr("\n", attr.clone());
    delta.insert("third");
    delta.insert_attr("\n", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading</p><ul><li>first</li><li>second</li><li>third</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    //error!("list_delete_test() tests 2");
    let mut delta = Delta::default();
    delta.retain(8);
    delta.delete(5);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading</p><ul><li><br></li><li>second</li><li>third</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    //error!("list_delete_test() tests 3");
    let mut delta = Delta::default();
    delta.retain(8); // cursor at empty <li>
    delta.delete(1);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading</p><ul><li>second</li><li>third</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    //error!("list_delete_test() tests 4");
    let mut delta = Delta::default();
    delta.retain(15); // cursor right before third
    delta.delete(6); // delete 5 characters and one LI

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading</p><ul><li>second</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    //error!("list_delete_test() tests 5");
    let mut delta = Delta::default();
    delta.retain(8);
    delta.delete(7);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading</p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn list_retain_3_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("list_retain_3_test");
    doc.append_to_body();

    let mut attr = Attributes::default();
    attr.insert(LIST_ATTR_KEY, LIST_BULLET);

    // Test 1
    //--------------------------------------------------------------------------
    //error!("list_retain_3_test - TEST 1");
    let mut delta = Delta::default();
    delta.insert("Leading\nfirst");
    delta.insert_attr("\n", attr.clone());
    delta.insert("second");
    delta.insert_attr("\n", attr.clone());
    delta.insert("third");
    delta.insert_attr("\n", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading</p><ul><li>first</li><li>second</li><li>third</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    // Test 2
    //--------------------------------------------------------------------------
    //error!("list_retain_3_test - TEST 2");
    let mut attr_n = Attributes::default();
    attr_n.insert(LIST_ATTR_KEY, AttrVal::Null);

    let mut delta = Delta::default();
    delta.retain(20);
    delta.retain_attr(1, attr_n.clone());

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect =
        r#"<p>Leading</p><ul><li>first</li></ul><p>second</p><ul><li>third</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn list_split_insert_in_the_middle_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("list_split_insert_in_the_middle_test");
    doc.append_to_body();

    let mut attr = Attributes::default();
    attr.insert(LIST_ATTR_KEY, LIST_BULLET);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("Leading text\nfirst");
    delta.insert_attr("\n", attr.clone());
    delta.insert("second");
    delta.insert_attr("\n", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading text</p><ul><li>first</li><li>second</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    //Insert a normal <p> right before the last <li>
    let mut delta = Delta::default();
    delta.retain(25);
    delta.insert("\n");

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading text</p><ul><li>first</li></ul><p>second</p><ul><li><br></li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn list_retain_2_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("list_retain_2_test");
    doc.append_to_body();

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("text\none\ntwo\n");

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p>text</p><p>one</p><p>two</p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let mut attr = Attributes::default();
    attr.insert(LIST_ATTR_KEY, LIST_BULLET);

    //--------------------------------------------------------------------------
    //error!("list_retain() - Test 1");
    let mut delta = Delta::default();
    delta.retain(8); // end of first LI
    delta.retain_attr(1, attr.clone());
    delta.retain(3); // end of first LI
    delta.retain_attr(1, attr.clone());

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p>text</p><ul><li>one</li><li>two</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn list_split_at_extremes_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("list_split_at_extremes_test");
    doc.append_to_body();

    let mut attr = Attributes::default();
    attr.insert(LIST_ATTR_KEY, LIST_BULLET);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("Leading text\nfirst");
    delta.insert_attr("\n", attr.clone());
    delta.insert("second");
    delta.insert_attr("\n", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading text</p><ul><li>first</li><li>second</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    //Insert a normal <p> right before the first <li>
    let mut delta = Delta::default();
    delta.retain(13); // cursor position ...<li>[*]first</li>...
    delta.insert("\n");

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect =
        r#"<p>Leading text</p><p><br></p><ul><li>first</li><li>second</li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    //Insert a normal <p> right after the second <li>-text
    let mut delta = Delta::default();
    delta.retain(26); // cursor position ...<li>second[*]</li>...
    delta.insert("\n"); // this will turn "second" in a <p> block !!

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p>Leading text</p><p><br></p><ul><li>first</li></ul><p>second</p><ul><li><br></li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn list_insert_in_between_p_blocks_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("list_insert_in_between_p_blocks_test");
    doc.append_to_body();
    doc.open()?;

    let mut attr = Attributes::default();
    attr.insert(LIST_ATTR_KEY, LIST_BULLET);

    // //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("\n");

    doc.open()?;
    doc.apply_delta(delta)?;
    doc.get_cursor().calculate_retain_index();

    let expect = r#"<p><br></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(1);
    delta.insert_attr("\n", attr.clone());

    doc.apply_delta(delta)?;
    doc.get_cursor().calculate_retain_index();

    assert_eq!(
        doc.get_cursor()
            .get_doc_node()
            .get_doc_dom_node()
            .get_node_name(),
        "P"
    );

    let expect = r#"<p><br></p><ul><li><br></li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(2);
    delta.insert_attr("\n", attr.clone());

    doc.apply_delta(delta)?;

    assert_eq!(
        doc.get_cursor()
            .get_doc_node()
            .get_doc_dom_node()
            .get_node_name(),
        "P"
    );

    let expect = r#"<p><br></p><ul><li><br></li><li><br></li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn list_insert_in_p_block_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("list_insert_in_p_block_test");
    doc.append_to_body();
    doc.open()?;

    let mut attr = Attributes::default();
    attr.insert(LIST_ATTR_KEY, LIST_BULLET);

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert_attr("\n", attr.clone());
    doc.apply_delta(delta)?;

    let expect = r#"<ul><li><br></li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    //--------------------------------------------------------------------------
    //Put cursor in LI block
    let cursor = doc.get_cursor();
    cursor.backspace()?;
    assert_eq!(
        doc.get_cursor()
            .get_doc_node()
            .get_doc_dom_node()
            .get_node_name(),
        "LI"
    );

    //--------------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert_attr("\n", attr);

    doc.apply_delta(delta)?;

    // No we do not do a back space, but we actually are in the 2nd <li> block
    assert_eq!(
        doc.get_cursor()
            .get_doc_node()
            .get_doc_dom_node()
            .get_node_name(),
        "LI"
    );

    let expect = r#"<ul><li><br></li><li><br></li></ul><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}
