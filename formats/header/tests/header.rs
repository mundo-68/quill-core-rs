use anyhow::Result;
use core_formats::format_const::{NAME_P_BLOCK, NAME_TEXT};
use core_formats::{P_FORMAT, TEXT_FORMAT};
use delta::attributes::Attributes;
use delta::delta::Delta;
use header::{HeaderBlock, NAME_HEADER};
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
        r.register_block_fmt(NAME_HEADER, Arc::new(HeaderBlock::new()))
            .unwrap();
        r.register_block_fmt(NAME_P_BLOCK, P_FORMAT.deref().clone())
            .unwrap();
        r.register_line_fmt(NAME_TEXT, TEXT_FORMAT.deref().clone())
            .unwrap();
        Mutex::new(1)
    });
}

#[test]
fn detect_hx_line_format() -> Result<()> {
    let mut attr = Attributes::default();
    attr.insert("heading", 2);

    let mut delta = Delta::default();
    delta.insert("This text is a normal first line \n This one will become a H2 header ");
    delta.insert_attr("\n", attr);

    let op = delta.get_ops_ref().get(1).unwrap();

    let hx_block = HeaderBlock::new();
    assert_eq!(hx_block.format_name(), NAME_HEADER);

    assert_eq!(hx_block.applies(op)?, true);
    Ok(())
}

#[wasm_bindgen_test]
fn header_format_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("header_format");
    doc.append_to_body();

    let mut delta = Delta::default();

    let mut attr = Attributes::default();
    attr.insert("heading", 2);
    delta.insert("This text is a normal first line \nThis one will become a H2 header ");
    delta.insert_attr("\n", attr);

    doc.open()?;
    doc.apply_delta(delta)?;

    let html_txt = r##"<p>This text is a normal first line </p><h2>This one will become a H2 header </h2><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);
    Ok(())
}

#[wasm_bindgen_test]
fn double_header_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("double_header_test");
    doc.append_to_body();

    let mut delta = Delta::default();

    let mut attr = Attributes::default();
    attr.insert("heading", 2);

    delta.insert("This text is a normal first line \nThis one will become a H2 header ");
    delta.insert_attr("\n", attr.clone());

    delta.insert("Next H2 header ");
    delta.insert_attr("\n", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let html_txt = r##"<p>This text is a normal first line </p><h2>This one will become a H2 header </h2><h2>Next H2 header </h2><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);
    Ok(())
}

#[wasm_bindgen_test]
fn formatted_header_format_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("formatted_header_format_test");
    doc.append_to_body();

    //-----------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("bold", true);

    let mut delta = Delta::default();
    delta.insert("This text is a normal first line \nThis one will become a H2 header ");
    delta.insert_attr("bold faced text", attr);

    let mut attr = Attributes::default();
    attr.insert("heading", 2);
    delta.insert_attr("\n", attr);

    //-----------------------------------------------------------------
    doc.open()?;
    doc.apply_delta(delta)?;

    //error!( "HTML composed = \n {}", dom_print::pretty_delimited_print( doc.get_root().get_element().unwrap(), true ) );
    let html_txt = r##"<p>This text is a normal first line </p><h2>This one will become a H2 header <strong>bold faced text</strong></h2><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);
    Ok(())
}

#[wasm_bindgen_test]
fn header_delete_text_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("header_delete_text_test");
    doc.append_to_body();

    //-----------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("heading", 2);

    let mut delta = Delta::default();
    delta.insert("H2 header");
    delta.insert_attr("\n", attr);

    doc.open()?;
    doc.apply_delta(delta)?;

    let html_txt = r##"<h2>H2 header</h2><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);

    //-----------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(3);
    delta.delete(6);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let html_txt = r##"<h2>H2 </h2><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);

    //-----------------------------------------------------------------
    let mut delta = Delta::default();
    delta.delete(3);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let html_txt = r##"<h2><br></h2><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);

    //-----------------------------------------------------------------
    let mut delta = Delta::default();
    delta.delete(1);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let html_txt = r##"<p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);
    Ok(())
}

#[wasm_bindgen_test]
fn header_delete_block_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("header_delete_block_test");
    doc.append_to_body();

    //-----------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("heading", 2);

    let mut delta = Delta::default();
    delta.insert("H2 header");
    delta.insert_attr("\n", attr);

    doc.open()?;
    doc.apply_delta(delta)?;

    let html_txt = r##"<h2>H2 header</h2><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);

    //-----------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(9);
    delta.delete(1);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let html_txt = r##"<p>H2 header</p>"##;
    assert_eq!(doc.as_html_string(), html_txt);
    Ok(())
}

#[wasm_bindgen_test]
fn header_delete_all_content_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("header_delete_all_content_test");
    doc.append_to_body();

    //-----------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("heading", 2);

    let mut delta = Delta::default();
    delta.insert("\nH2 header");
    delta.insert_attr("\n", attr);

    doc.open()?;
    doc.apply_delta(delta)?;

    let html_txt = r##"<p><br></p><h2>H2 header</h2><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);

    //-----------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(1); //skip <p>
    delta.delete(9);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let html_txt = r##"<p><br></p><h2><br></h2><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);
    Ok(())
}
