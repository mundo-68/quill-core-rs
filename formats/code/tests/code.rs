use anyhow::Result;
use code::{CodeBlock, NAME_CODE};
use core_formats::format_const::{NAME_P_BLOCK, NAME_TEXT};
use core_formats::{P_FORMAT, TEXT_FORMAT};
use delta::attributes::Attributes;
use delta::delta::Delta;
use delta::types::attr_val::AttrVal;
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
        r.register_block_fmt(NAME_CODE, Arc::new(CodeBlock::new()))
            .unwrap();
        r.register_block_fmt(NAME_P_BLOCK, P_FORMAT.deref().clone())
            .unwrap();
        r.register_line_fmt(NAME_TEXT, TEXT_FORMAT.deref().clone())
            .unwrap();
        Mutex::new(1)
    });
}

#[wasm_bindgen_test]
fn code_format_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("code_format_test");
    doc.append_to_body();

    let mut attr = Attributes::default();
    attr.insert("code-block", true);

    let mut delta = Delta::default();
    delta.insert("hello ");
    delta.insert_attr("\n", attr.clone());
    delta.insert("sweet");
    delta.insert_attr("\n", attr.clone());
    delta.insert("world");
    delta.insert_attr("\n", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let html_txt = r##"<span class="ql-pre">hello </span><span class="ql-pre">sweet</span><span class="ql-pre">world</span><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);
    Ok(())
}

#[wasm_bindgen_test]
fn code_format_another_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("code_format_another_test");
    doc.append_to_body();

    let mut attr = Attributes::default();
    attr.insert("code-block", true);

    let mut delta = Delta::default();
    delta.insert("hello    4spaces world");
    delta.insert_attr("\n", attr.clone());
    delta.insert("next line with 4 trailing spaces    ");
    delta.insert_attr("\n", attr.clone());
    delta.insert("last line");
    delta.insert_attr("\n", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    //error!( "HTML composed = \n {}", dom_print::pretty_delimited_print( doc.get_root().get_node(), true ) );
    let html_txt = r##"<span class="ql-pre">hello    4spaces world</span><span class="ql-pre">next line with 4 trailing spaces    </span><span class="ql-pre">last line</span><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);
    Ok(())
}

#[wasm_bindgen_test]
fn code_delete_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("code_delete_test");
    doc.append_to_body();

    //-----------------------------------------------------------------------
    //error!("code_delete_test() - Test 1");
    let mut delta = Delta::default();

    delta.insert("code text ");

    let mut attr = Attributes::default();
    attr.insert("code-block".to_string(), AttrVal::Bool(true));
    delta.insert_attr("\n", attr);

    doc.open()?;
    doc.apply_delta(delta)?;

    let html_txt = r##"<span class="ql-pre">code text </span><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);

    //-----------------------------------------------------------------------
    //error!("code_delete_test() - Test 2");
    let mut delta = Delta::default();
    delta.retain(4);
    delta.delete(5);
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let html_txt = r##"<span class="ql-pre">code </span><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);

    //-----------------------------------------------------------------------
    //error!("code_delete_test() - Test 3");
    let mut delta = Delta::default();
    delta.delete(5);
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let html_txt = r##"<span class="ql-pre"><br></span><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);

    //-----------------------------------------------------------------------
    //error!("code_delete_test() - Test 4");
    let mut delta = Delta::default();
    delta.delete(1);
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let html_txt = r##"<p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);
    Ok(())
}

#[wasm_bindgen_test]
fn code_double_span_delete_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("code_double_span_delete_test");
    doc.append_to_body();

    //-----------------------------------------------------------------------
    let mut delta = Delta::default();

    let mut attr = Attributes::default();
    attr.insert("code-block", true);

    delta.insert("code text 1");
    delta.insert_attr("\n", attr.clone());

    delta.insert("code text 2");
    delta.insert_attr("\n", attr.clone());

    doc.open()?;
    doc.apply_delta(delta)?;

    let html_txt = r##"<span class="ql-pre">code text 1</span><span class="ql-pre">code text 2</span><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);

    //-----------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(4);
    delta.delete(5);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let html_txt =
        r##"<span class="ql-pre">code 1</span><span class="ql-pre">code text 2</span><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);

    //-----------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain(6);
    delta.delete(1);
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let html_txt = r##"<span class="ql-pre">code 1code text 2</span><p><br></p>"##;
    assert_eq!(doc.as_html_string(), html_txt);
    Ok(())
}
