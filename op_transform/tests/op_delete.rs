#[path = "op_transform_test_utils.rs"]
mod op_transform_test_utils;

use crate::op_transform_test_utils::create_text;
use anyhow::Result;
use delta::attributes::Attributes;
use delta::delta::Delta;
use delta::operations::DeltaOperation;
use node_tree::cursor::LocationIdentifyer;
use op_transform::doc_root::DocumentRoot;
use op_transform::op_delete;
use op_transform::registry::init_test_registry;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn op_delete_start_test() -> Result<()> {
    init_test_registry();
    let doc = DocumentRoot::new("op_delete_start_test");
    doc.append_to_body();
    create_text(&doc)?;

    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let first = 0;
    let cursor = doc.get_cursor();
    let p = doc.get_root().get_children().get(first).unwrap().clone();
    cursor.set_before(&p.get_children().get(0).unwrap().clone());

    //--------------------------------------------------------------------------
    let r = cursor.get_retain_index();
    assert_eq!(r, 0);
    assert_eq!(
        cursor.get_location().get_location(),
        LocationIdentifyer::Before
    );

    //--------------------------------------------------------------------------
    op_delete::delete(&cursor, 4)?;
    let expect = r#"<p>_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let r = cursor.get_retain_index();
    assert_eq!(r, 0);
    assert_eq!(
        cursor.get_location().get_location(),
        LocationIdentifyer::Before
    );

    //--------------------------------------------------------------------------
    op_delete::delete(&cursor, 4)?;
    let expect = r#"<p><strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let r = cursor.get_retain_index();
    assert_eq!(r, 0);
    assert_eq!(
        cursor.get_location().get_location(),
        LocationIdentifyer::Before
    );
    Ok(())
}

#[wasm_bindgen_test]
fn op_delete_ending_test() -> Result<()> {
    init_test_registry();
    let doc = DocumentRoot::new("op_delete_ending_test");
    doc.append_to_body();
    create_text(&doc)?;

    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //--------------------------------------------------------------------------
    let last = 2;
    let cursor = doc.get_cursor();
    let p = doc.get_root().get_children().get(last).unwrap().clone();
    cursor.set_at(&p, 0);
    let result = op_delete::delete(&cursor, 1);
    assert!(result.is_err()); //Error::DeletingLastBlock

    //THIS IS WRONG --> WE SHOULD AND CAN NOT DELETE THE LAST <p> BLOCK IN A DOCUMENT
    //let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p>"#;
    //test_match(doc.get_root().get_html_node(), expect);

    let r = cursor.get_retain_index();
    assert_eq!(r, 42);
    assert_eq!(
        cursor.get_location().get_location(),
        LocationIdentifyer::After
    );
    Ok(())
}

#[wasm_bindgen_test]
fn op_delete_middle_test() -> Result<()> {
    init_test_registry();
    let doc = DocumentRoot::new("op_delete_middle_test");
    doc.append_to_body();

    create_text(&doc)?;

    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let first = 0;
    let cursor = doc.get_cursor();
    let p = doc.get_root().get_children().get(first).unwrap().clone();
    let strong = p.get_children().get(1).unwrap().clone();
    cursor.set_before(&strong);

    //--------------------------------------------------------------------------
    op_delete::delete(&cursor, 4)?;
    let expect = r#"<p>TEXT_1_1<strong>_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let r = cursor.get_retain_index();
    assert_eq!(r, 8);
    assert_eq!(
        cursor.get_location().get_location(),
        LocationIdentifyer::Before
    );

    //--------------------------------------------------------------------------
    op_delete::delete(&cursor, 4)?;
    let expect = r#"<p>TEXT_1_1TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let r = cursor.get_retain_index();
    assert_eq!(r, 8);
    assert_eq!(cursor.get_location().get_location(), LocationIdentifyer::At);
    assert_eq!(cursor.get_location().index().unwrap(), 8);
    Ok(())
}

#[wasm_bindgen_test]
fn op_delete_block_node_test() -> Result<()> {
    init_test_registry();
    let doc = DocumentRoot::new("op_delete_block_node_test");
    doc.append_to_body();

    create_text(&doc)?;

    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    let first = 0;
    let cursor = doc.get_cursor();
    let p = doc.get_root().get_children().get(first).unwrap().clone();
    let last = p.get_children().get(2).unwrap().clone();
    cursor.set_after(&last);
    //error!("Before: {}", &cursor);

    //--------------------------------------------------------------------------
    op_delete::delete(&cursor, 1)?;
    let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3TEXT_2_1<strong>TEXT_2_2</strong></p><p><br></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //error!("After: {}", cursor);
    let r = cursor.get_retain_index();
    assert_eq!(r, 24);
    assert_eq!(cursor.get_location().get_location(), LocationIdentifyer::At);
    assert_eq!(cursor.get_location().index().unwrap(), 8);
    Ok(())
}

#[wasm_bindgen_test]
fn op_delete_backspace_formatted_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("op_delete_backspace_formatted_test");
    doc.append_to_body();
    doc.open()?;

    let mut delta = Delta::default();
    let mut attr = Attributes::default();
    attr.insert("bold", true);
    delta.insert_attr("Hello", attr);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    //let printer = HtmlPrinter::new(&doc);
    //error!("op_delete_backspace_formatted_test()\n{}", printer.print_one_liner());

    let op = DeltaOperation::delete(1);
    for _i in 0..5 {
        doc.get_cursor().backspace()?; //back 1 position before deleting 1
        doc.apply_operation(op.clone())?;
    }

    let expect = "<p><br></p>";
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}
