#[path = "op_transform_test_utils.rs"]
mod op_transform_test_utils;

use anyhow::Result;
use delta::attributes::Attributes;
use delta::delta::Delta;
use op_transform::doc_root::DocumentRoot;
use op_transform::registry::init_test_registry;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn apply_insert_format_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("op_insert_multi_p_test");
    doc.append_to_body();

    //------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("\n\n\n");
    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p><p><br></p><p><br></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("bold", true);

    let mut delta = Delta::default();
    delta.retain(1);
    delta.insert_attr("b", attr);

    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p><p><strong>b</strong></p><p><br></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("bold", true);

    let mut delta = Delta::default();
    delta.retain(2);
    delta.insert_attr("a", attr);

    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p><p><strong>ba</strong></p><p><br></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}
