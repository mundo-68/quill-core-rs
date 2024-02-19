use anyhow::Result;
use core_formats::format_const::{NAME_P_BLOCK, NAME_TEXT};
use core_formats::{P_FORMAT, TEXT_FORMAT};
use delta::attributes::Attributes;
use delta::delta::Delta;
use delta::document::Document;
use delta::operations::DeltaOperation;
use delta::types::attr_val::AttrVal;
use link::{LinkFormat, NAME_LINK};
use node_tree::cursor::Cursor;
use node_tree::format_trait::FormatTait;
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
        r.register_block_fmt(NAME_P_BLOCK, P_FORMAT.deref().clone())
            .unwrap();
        r.register_line_fmt(NAME_LINK, Arc::new(LinkFormat::new()))
            .unwrap();
        r.register_line_fmt(NAME_TEXT, TEXT_FORMAT.deref().clone())
            .unwrap();
        Mutex::new(1)
    });
}

static LINK_ATTR: &'static str = "link"; //marker attribute to recognize this format

fn create_test_link(doc: &mut DocumentRoot) -> Result<()> {
    let mut delta = Delta::default();

    let mut attr1 = Attributes::default();
    attr1.insert("link", "https://");
    delta.insert_attr("go", attr1);

    let mut attr2 = Attributes::default();
    attr2.insert("link", "https://");
    attr2.insert("bold", true);
    delta.insert_attr("og", attr2);

    let mut attr3 = Attributes::default();
    attr3.insert("link", "https://");
    delta.insert_attr("le", attr3);

    //error!("create_test_link() document = {}", &delta);

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p><a href="https://">go<strong>og</strong>le</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn link_create_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new(&*"link_create_test");
    doc.append_to_body();

    //---------------------------------------------------------------------
    //{ insert: "Google", attributes: { link: 'https://www.google.com' } }
    let mut delta = Delta::default();
    let mut attr = Attributes::default();
    attr.insert("link", "https://");
    delta.insert_attr("google", attr);

    doc.open()?;
    doc.apply_delta(delta)?;

    // let s = pretty_print(doc.get_root().get_dom_element().unwrap().as_ref(), true);
    // error!( "link {}", s );

    let len = Delta::document_length(&doc.to_delta());
    assert_eq!(len, 7);

    let expect = r#"<p><a href="https://">google</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //---------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(2);

    doc.apply_delta(delta)?;
    Ok(())
}

#[wasm_bindgen_test]
fn link_insert_front_end_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new(&*"link_insert_front_end_test");
    doc.append_to_body();

    //---------------------------------------------------------------------
    //{ insert: "Google", attributes: { link: 'https://www.google.com' } }
    let mut delta = Delta::default();
    let mut attr = Attributes::default();
    attr.insert("link", "https://");
    delta.insert_attr("oo", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let len = Delta::document_length(&doc.to_delta());
    assert_eq!(len, 3);

    let expect = r#"<p><a href="https://">oo</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //---------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(2);
    delta.insert_attr("gle", attr.clone());

    doc.reset_cursor();
    doc.apply_delta(delta)?;
    let expect = r#"<p><a href="https://">oogle</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //---------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert_attr("g", attr.clone());

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p><a href="https://">google</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn link_formatted_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new(&*"link_formatted_test");
    doc.append_to_body();

    let mut attr1: Attributes = Attributes::default();
    attr1.insert("link", "https://");

    let mut attr2 = Attributes::default();
    attr2.insert("link", "https://");
    attr2.insert("bold", true);

    //-------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert_attr("go", attr1.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p><a href="https://">go</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //-------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(2);
    delta.insert_attr("og", attr2.clone());

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p><a href="https://">go<strong>og</strong></a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //-------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(4);
    delta.insert_attr("le", attr1.clone());

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p><a href="https://">go<strong>og</strong>le</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //-------------------------------------------------------------
    let mut doc = DocumentRoot::new(&*"link_formatted_test_V2");
    doc.append_to_body();

    create_test_link(&mut doc)?;

    let expect = r#"<p><a href="https://">go<strong>og</strong>le</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn link_split_text_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new(&*"link_formatted_test");
    doc.append_to_body();

    create_test_link(&mut doc)?;

    let expect = r#"<p><a href="https://">go<strong>og</strong>le</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //root length --> just the operation !!
    let root = doc.get_root().clone();
    assert_eq!(root.get_operation().op_len(), 0);

    let cursor = Cursor::new();
    //===========================================================================
    let p = root.get_children().get(0).unwrap().clone();
    assert_eq!(p.get_operation().op_len(), 1);

    let a = p.get_children().get(0).unwrap().clone();
    assert_eq!(a.get_operation().op_len(), 0);

    assert_eq!(a.child_count(), 3);

    cursor.set_before(&a.get_children().get(0).unwrap().clone());
    cursor.get_doc_node().get_formatter().split_leaf(&cursor)?;

    //===========================================================================
    let p = root.get_children().get(0).unwrap().clone();
    assert_eq!(p.get_operation().op_len(), 1);

    let a = p.get_children().get(0).unwrap().clone();
    assert_eq!(a.get_operation().op_len(), 0);

    assert_eq!(a.child_count(), 3);

    cursor.set_after(&a.get_children().get(1).unwrap().clone());
    cursor.get_doc_node().get_formatter().split_leaf(&cursor)?;

    Ok(())
}

#[wasm_bindgen_test]
fn link_delete_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new(&*"link_delete_test");
    doc.append_to_body();
    create_test_link(&mut doc)?;
    doc.reset_cursor();

    //====================================================================
    let mut delta = Delta::default();
    delta.retain(2);
    delta.delete(1);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p><a href="https://">go<strong>g</strong>le</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let p = doc.get_root().get_children().get(0).unwrap().clone();
    let a = p.get_children().get(0).unwrap().clone();
    assert_eq!(a.child_count(), 3);

    //====================================================================
    let mut delta = Delta::default();
    delta.delete(2);
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p><a href="https://"><strong>g</strong>le</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let p = doc.get_root().get_children().get(0).unwrap().clone();
    let a = p.get_children().get(0).unwrap().clone();
    assert_eq!(a.child_count(), 2);

    //====================================================================
    let mut delta = Delta::default();
    delta.retain(1);
    delta.delete(2);
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let p = doc.get_root().get_children().get(0).unwrap().clone();
    let a = p.get_children().get(0).unwrap().clone();
    assert_eq!(a.child_count(), 1);

    let expect = r#"<p><a href="https://"><strong>g</strong></a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //====================================================================
    let mut delta = Delta::default();
    delta.delete(1);
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r#"<p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn link_insert_in_the_middle() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("link_insert_in_the_middle");
    doc.append_to_body();

    //CREATE DELTA
    //--------------------------------------------------------------------
    let mut delta = Delta::default();

    let mut attr1 = Attributes::default();
    attr1.insert(LINK_ATTR, AttrVal::from("HTTPS:://"));
    attr1.insert("bold".to_string(), AttrVal::from(true));
    delta.insert_attr("gole", attr1);

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p><a href="HTTPS:://"><strong>gole</strong></a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------
    let mut attr3 = Attributes::default();
    attr3.insert(LINK_ATTR, AttrVal::from("HTTPS:://"));
    attr3.insert("italic", true);

    let mut delta = Delta::default();
    delta.retain(2);
    delta.insert_attr("og", attr3);
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect =
        r#"<p><a href="HTTPS:://"><strong>go</strong><em>og</em><strong>le</strong></a></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn link_split_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("link_split_test");
    doc.append_to_body();

    create_test_link(&mut doc)?;

    //-------------------------------------------------------------------
    //bring cursor to BEFORE 3rd character goo[*]gle
    let mut delta = Delta::default();
    delta.retain(3);

    doc.reset_cursor();
    doc.apply_delta(delta)?;
    let cursor = doc.get_cursor();
    let idx = cursor.get_location().index().unwrap();
    let cursor_c = cursor
        .get_doc_node()
        .get_operation()
        .insert_value()
        .str_val()?
        .chars()
        .nth(idx)
        .unwrap();
    assert_eq!(cursor_c, 'g');

    let format: Arc<dyn FormatTait + Send + Sync> = Arc::new(LinkFormat::new());
    format.split_leaf(doc.get_cursor())?;

    // Isolated the 2nd 'o': [go][o][gle]
    let expect = r#"<p><a href="https://">go</a><a href="https://"><strong>o</strong></a><a href="https://"><strong>g</strong>le</a></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn link_formatted_prepend_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("link_formatted_prepend_test");
    doc.append_to_body();

    //CREATE DELTA
    //--------------------------------------------------------------------
    let mut delta = Delta::default();

    let mut attr = Attributes::default();
    attr.insert(LINK_ATTR, "http:");
    attr.insert("bold", true);
    delta.insert_attr("og", attr);

    let mut attr = Attributes::default();
    attr.insert(LINK_ATTR, "http:");
    delta.insert_attr("le", attr);

    //Create the basic document to have some structure to insert before
    doc.open()?;
    doc.apply_delta(delta)?;
    //we now have the "ogle" string without the "go"

    //--------------------------------------------------------------------
    let expect = r##"<p><a href="http:"><strong>og</strong>le</a></p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------
    let mut delta = Delta::default();

    let mut attr = Attributes::default();
    attr.insert(LINK_ATTR, "http:");
    delta.insert_attr("go", attr);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r##"<p><a href="http:">go<strong>og</strong>le</a></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn link_split_and_merge_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("link_split_and_merge_test");
    doc.append_to_body();

    let mut delta = Delta::default();

    let mut attr1 = Attributes::default();
    attr1.insert(LINK_ATTR, AttrVal::from("http:"));
    delta.insert_attr("pre", attr1);

    let mut attr2 = Attributes::default();
    attr2.insert(LINK_ATTR, AttrVal::from("http:"));
    attr2.insert("bold".to_string(), AttrVal::from(true));
    delta.insert_attr("gole", attr2);

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p><a href="http:">pre<strong>gole</strong></a></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //-------------------------------------------------------------------
    let format: Arc<dyn FormatTait + Send + Sync> = Arc::new(LinkFormat::new());
    doc.cursor_to_start();
    for i in 0 as usize..doc.to_delta().document_length() {
        doc.cursor_to_start();
        doc.apply_operation(DeltaOperation::retain(i))?;
        format.split_leaf(&doc.get_cursor())?;
        format.try_merge(doc.get_cursor(), &doc.get_cursor().get_doc_node())?;
        assert_eq!(doc.as_html_string(), expect);
    }
    Ok(())
}
