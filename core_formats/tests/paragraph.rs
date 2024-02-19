use anyhow::Result;
use core_formats::format_const::NAME_P_BLOCK;
use delta::delta::Delta;
use delta::operations::DeltaOperation;
use op_transform::doc_root::DocumentRoot;
use op_transform::registry::{init_test_registry, Registry};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// The paragraph formatter module does not add `<br/>`.
/// But the op_transform module used here does. So here we do  add it to the tests

#[wasm_bindgen_test]
fn test_block_format() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("test_block_format");
    doc.append_to_body();
    doc.open()?;

    let registry = Registry::get_ref()?;

    let op = DeltaOperation::insert("\n");
    assert_eq!(op.op_len(), 1);

    assert!(registry.is_block_fmt(&op)?);
    let format = registry.block_format(&op)?;
    assert_eq!(format.format_name(), NAME_P_BLOCK);

    //let op = DeltaOperation::insert("hi \ndi-hi");
    let op = DeltaOperation::insert("\n");

    assert!(registry.is_block_fmt(&op)?);
    let format = registry.block_format(&op)?;
    assert_eq!(format.format_name(), NAME_P_BLOCK);
    Ok(())
}

#[wasm_bindgen_test]
fn p_block_text_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("p_block_text_test");
    doc.append_to_body();
    doc.open()?;

    //------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("Hello World");
    doc.apply_delta(delta)?;
    let expect = r##"<p>Hello World</p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------------------
    doc.close();
    let expect = r##""##;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------------------
    doc.open()?;
    let mut delta = Delta::default();
    delta.insert("Hello\n World");
    doc.apply_delta(delta)?;
    let expect = r##"<p>Hello</p><p> World</p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------------------
    doc.close();
    let expect = r##""##;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------------------
    doc.open()?;
    let mut delta = Delta::default();
    delta.insert("Hello\n\n World");
    doc.apply_delta(delta)?;

    let expect = r##"<p>Hello</p><p><br></p><p> World</p>"##;
    assert_eq!(doc.as_html_string(), expect);

    let mut delta = Delta::default();
    delta.retain(5);
    delta.insert("\n");
    doc.reset_cursor();
    doc.apply_delta(delta)?;
    let expect = r##"<p>Hello</p><p><br></p><p><br></p><p> World</p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn p_block_formatted_end_text_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("p_block_formatted_end_text_test");
    doc.append_to_body();
    doc.open()?;

    let delta_str = r##"{"ops":[
            {"attributes": {"bold": true}, "insert": "bold"},
            {"insert": "fluts\n This text is \n centered"},
            {"attributes": {"align": "center"}, "insert": "\n"}
            ]}"##;
    //{"attributes": {"align": "center"}, "insert": "\n"},
    let delta: Delta = serde_json::from_str(delta_str).unwrap();
    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r##"<p><strong>bold</strong>fluts</p><p> This text is </p><p class="ql-align-center"> centered</p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn p_block_line_formatted_start_text() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("p_block_line_formatted_start_text");
    doc.append_to_body();
    doc.open()?;

    let delta_str = r##"{"ops":[
            {"insert": "centered"},
            {"attributes": {"align": "center"}, "insert": "\n"},
            {"attributes": {"bold": true}, "insert": "bold"},
            {"insert": "\n"}
            ]}"##;

    let delta: Delta = serde_json::from_str(delta_str).unwrap();
    doc.apply_delta(delta)?;

    let expect =
        r##"<p class="ql-align-center">centered</p><p><strong>bold</strong></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn p_block_new_doc() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("p_block_new_doc");
    doc.append_to_body();
    doc.open()?;

    let mut delta: Delta = Delta::default();
    doc.open()?;
    doc.apply_delta(delta.clone())?; // EMPTY delta !!

    let expect = r##"<p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);

    delta.insert("\n");
    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}
