use anyhow::Result;
use delta::attributes::Attributes;
use delta::delta::Delta;
use delta::types::attr_val::AttrVal;
use op_transform::doc_root::DocumentRoot;
use op_transform::registry::init_test_registry;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// The text_formatter module does not add `<br/>`.
/// But the op_transform module used here does. So here we do add it to the tests

#[wasm_bindgen_test]
fn apply_text_format_test() -> Result<()> {
    init_test_registry();
    init_test_registry();
    let mut doc = DocumentRoot::new("apply_text_format_test");
    doc.append_to_body();
    doc.open()?;

    let mut attr = Attributes::default();
    attr.insert("bold", true);

    //--------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("TEXT_1_1");
    // let t = t_format.create(delta, t_format.clone())?;
    // append(&par, t.clone());
    doc.apply_delta(delta)?;

    let expect = r#"<p>TEXT_1_1</p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------
    let mut delta = Delta::default();
    delta.retain_attr(8, attr.clone());
    doc.apply_delta(delta)?;
    let expect = r#"<p><strong>TEXT_1_1</strong></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------
    let mut drop_a = Attributes::default();
    drop_a.insert("bold", AttrVal::Null);
    let mut delta = Delta::default();
    delta.retain_attr(8, drop_a);
    doc.apply_delta(delta)?;
    let expect = r#"<p>TEXT_1_1</p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------
    attr.insert("italic", true);
    let mut delta = Delta::default();
    delta.retain_attr(8, attr);
    doc.apply_delta(delta)?;
    let expect = r#"<p><em><strong>TEXT_1_1</strong></em></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}
