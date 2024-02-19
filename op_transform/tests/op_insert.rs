#[path = "op_transform_test_utils.rs"]
mod op_transform_test_utils;

use anyhow::Result;
use delta::attributes::Attributes;
use delta::delta::Delta;
use op_transform::doc_root::DocumentRoot;
use op_transform::registry::init_test_registry;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;

#[wasm_bindgen_test]
fn op_insert_multi_p_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("op_insert_multi_p_test");
    doc.append_to_body();

    //------------------------------------------------------
    let mut delta = Delta::default();
    delta.insert("\n\n\n");
    delta.insert("\n");
    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p><p><br></p><p><br></p><p><br></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn op_insert_p_in_text_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("op_insert_p_in_text_test");
    doc.append_to_body();

    //------------------------------------------------------
    //error!("op_insert_p_in_text_test()  - 1 " );
    let mut delta = Delta::default();
    delta.insert("Hello World");
    //delta.insert("\n");

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r##"<p>Hello World</p>"##;
    assert_eq!(doc.as_html_string(), expect);

    let mut delta = Delta::default();
    delta.retain(5);
    delta.insert("\n");

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r##"<p>Hello</p><p> World</p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------------------
    //error!("op_insert_p_in_text_test()  - 2 " );
    let mut delta = Delta::default();
    delta.insert("\n");

    doc.reset_cursor();
    doc.apply_delta(delta)?;
    let expect = r##"<p><br></p><p>Hello</p><p> World</p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------------------
    //error!("op_insert_p_in_text_test()  - 3 " );
    let mut delta = Delta::default();
    delta.retain(13);
    delta.insert("\n");
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p><p>Hello</p><p> World</p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn op_insert_multi_text_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("op_insert_multi_text_test");
    doc.append_to_body();

    // Concatenation attributed and non attributed text
    //------------------------------------------------------
    //error!("op_insert_multi_text_test TEST 1 ");
    let mut b = Attributes::default();
    b.insert("bold", true);

    let mut e = Attributes::default();
    e.insert("italic", true);

    let mut delta = Delta::default();
    delta.insert_attr("Hello", b.clone());
    delta.insert(" ");
    delta.insert_attr("World", e.clone());
    delta.insert("\n");

    doc.open()?;
    doc.apply_delta(delta)?;

    let expect = r##"<p><strong>Hello</strong> <em>World</em></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //Insert multiple empty paragraphs
    //------------------------------------------------------
    //error!("op_insert_multi_text_test TEST 2 ");
    let mut delta = Delta::default();
    delta.insert("\n\n");
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect =
        r##"<p><br></p><p><br></p><p><strong>Hello</strong> <em>World</em></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //Insert text in empty paragraph
    //------------------------------------------------------
    //error!("op_insert_multi_text_test TEST 3 ");
    let mut delta = Delta::default();
    delta.retain(1);
    delta.insert("q");
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p><p>q</p><p><strong>Hello</strong> <em>World</em></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //Insert text in formatted text
    //------------------------------------------------------
    //error!("op_insert_multi_text_test TEST 4 ");
    let mut delta = Delta::default();
    delta.retain(8);
    delta.insert("r");
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p><p>q</p><p><strong>Hello</strong>r <em>World</em></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //Insert un-formatted text inbetween formatted text
    //------------------------------------------------------
    //error!("op_insert_multi_text_test TEST 5 ");
    let mut delta = Delta::default();
    delta.retain(5);
    delta.insert("s");
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p><p>q</p><p><strong>He</strong>s<strong>llo</strong>r <em>World</em></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);

    //Insert formatted text inbetween formatted text
    //------------------------------------------------------
    //error!("op_insert_multi_text_test TEST 6 ");
    let mut delta = Delta::default();
    delta.retain(4);
    delta.insert_attr("t", b.clone());
    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p><p>q</p><p><strong>Hte</strong>s<strong>llo</strong>r <em>World</em></p><p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}
