#[path = "op_transform_test_utils.rs"]
mod op_transform_test_utils;

use anyhow::Result;
use delta::attributes::Attributes;
use delta::delta::Delta;
use delta::types::attr_val::AttrVal::Null;
use op_transform::doc_root::DocumentRoot;
use op_transform::registry::init_test_registry;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn retain_formatted_test() -> Result<()> {
    init_test_registry();

    let mut doc = DocumentRoot::new("retain_formatted_test");
    doc.append_to_body();
    doc.open()?;

    let mut delta = Delta::default();
    delta.insert("This text is used as tests text");
    doc.apply_delta(delta)?;

    let expect = r#"<p>This text is used as tests text</p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let mut delta = Delta::default();
    delta.retain(31);
    delta.insert("\n");
    doc.apply_delta(delta)?;

    //--------------------------------------------------------------------------
    // Test 1: Check tests text
    //--------------------------------------------------------------------------
    let expect = r#"<p>This text is used as tests text</p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    // Test 2: make partially bold
    //--------------------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("bold", true);

    let mut delta = Delta::default();
    delta.retain(5);
    delta.retain_attr(4, attr.clone());

    doc.cursor_to_start();
    doc.apply_delta(delta)?;

    let expect = r#"<p>This <strong>text</strong> is used as tests text</p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    // Test 3: make partially italic
    //--------------------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("italic", true);

    let mut delta = Delta::default();
    delta.retain(9);
    delta.retain_attr(8, attr.clone());

    doc.cursor_to_start();
    doc.apply_delta(delta)?;

    let expect = r#"<p>This <strong>text</strong><em> is used</em> as tests text</p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    // Test 4: Make small
    //--------------------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("small", true);

    let mut delta = Delta::default();
    delta.retain_attr(5, attr.clone());

    doc.cursor_to_start();
    doc.apply_delta(delta)?;

    let expect = r#"<p><small>This </small><strong>text</strong><em> is used</em> as tests text</p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    // Test 4: Make deleted
    //--------------------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("deleted", true);
    attr.insert("small", Null);

    let mut delta = Delta::default();
    delta.retain_attr(5, attr.clone());

    let mut attr = Attributes::default();
    attr.insert("deleted", true);
    attr.insert("bold", Null);

    delta.retain_attr(2, attr.clone());

    doc.cursor_to_start();
    doc.apply_delta(delta)?;

    let expect =
        r#"<p><del>This te</del><strong>xt</strong><em> is used</em> as tests text</p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn retain_formatted_block_test() -> Result<()> {
    init_test_registry();

    let mut doc = DocumentRoot::new("retain_formatted_block_test");
    doc.append_to_body();
    doc.open()?;

    let mut delta = Delta::default();
    delta.insert("Hello World\n\n");

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r#"<p>Hello World</p><p><br></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //-----------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("align", "center");

    // Note that if we first retain, and then insert(centered) we get the centered
    // we get the `centered` text in the last <p> node, since we only add the attribute
    // to the first <p> and then skip to the next (last) <p> node
    let mut delta = Delta::default();
    doc.cursor_to_start();
    delta.retain(12); //After first <p>
    delta.insert("centered");
    delta.retain_attr(1, attr);

    doc.apply_delta(delta)?;

    let expect = r#"<p>Hello World</p><p class="ql-align-center">centered</p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}
