#[path = "op_transform_test_utils.rs"]
mod op_transform_test_utils;

use delta::attributes::Attributes;
use delta::delta::Delta;
use delta::document::Document;
use delta::operations::DeltaOperation;
use node_tree::cursor::cursor_points_to;
use op_transform::doc_root::DocumentRoot;
use op_transform::registry::init_test_registry;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn retain_length_test() -> anyhow::Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("retain_length_test");
    doc.append_to_body();

    //-----------------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("bold", true);

    let mut delta = Delta::default();
    delta.insert("\nabcdefghij");
    delta.insert_attr("zyxwv\nusrqp", attr);
    delta.insert("\n");
    delta.insert("0123456789");

    doc.open()?;
    doc.apply_delta(delta)?;

    //doc text with bock nodes replaced by @
    let str = "@abcdefghijzyxwv@usrqp@0123456789@";

    // -----------------------------------------------------------------------
    // let html = pretty_print(doc.get_root().get_dom_element().unwrap(), true);
    // error!( "{}", html );

    doc.cursor_to_end();
    //error!( "{}", &cursor );
    assert_eq!(doc.get_cursor().get_retain_index(), 33 as usize);

    for i in 0..Delta::document_length(&doc.to_delta()) {
        doc.reset_cursor();
        let c = str.chars().nth(i).unwrap();
        doc.apply_operation(DeltaOperation::retain(i))?;
        //error!( "char = {}", c);
        //error!( "cursor points to char = {}", points_to(doc.get_cursor()));
        //error!( "cursor points to = {}", doc.get_cursor());
        if c.to_string() == "@".to_string() {
            assert_eq!(cursor_points_to(doc.get_cursor()), "<P>");
        } else {
            assert_eq!(cursor_points_to(doc.get_cursor()), c.to_string())
        }
    }
    Ok(())
}

#[wasm_bindgen_test]
fn retain_length_less_simple_test() -> anyhow::Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("retain_length_less_simple_test");
    doc.append_to_body();

    //-----------------------------------------------------------------------
    let mut attr = Attributes::default();
    attr.insert("bold", true);

    let mut delta = Delta::default();
    delta.insert("abcdefghij");
    delta.insert_attr("zyxwv\n\n\nusrqp", attr);
    delta.insert("\n");
    delta.insert("0123456789");
    doc.close();
    doc.open()?;
    doc.apply_delta(delta)?;

    //Check string of what is expected to be pointed at
    //Adding '@' as marker for <P> blocks
    let txt = "abcdefghijzyxwv@@@usrqp@0123456789@";

    //-----------------------------------------------------------------------
    // let html = pretty_print(doc.get_root().get_dom_element().unwrap(), true);
    // error!( "{}", html );

    //-----------------------------------------------------------------------
    //error!( "Doc node structure ... {}", print_doc_node_intern(doc.get_root(), "".to_string()) );

    let length = 35 as usize;
    assert_eq!(doc.get_cursor().get_retain_index(), length - 1);
    assert_eq!(Delta::document_length(&doc.to_delta()), length); //includes last <P>

    doc.reset_cursor();
    //error!( "Starting position of the cursor ...{}", &cursor );

    for i in 0..length - 1 {
        //error!( "retain counter = {}", i );
        //error!( "space cursor = {}", doc.get_cursor() );
        //error!( "retain index = {}", doc.get_cursor().get_retain_index());
        assert_eq!(doc.get_cursor().get_retain_index(), i);

        let c = txt.chars().nth(i).unwrap();
        //error!( "expected char = {}", c);
        //error!( "cursor points to char = {}", points_to(doc.get_cursor()));
        //error!( "cursor = {}", doc.get_cursor());
        if c.to_string() == "@".to_string() {
            assert_eq!(cursor_points_to(doc.get_cursor()), "<P>");
        } else {
            assert_eq!(cursor_points_to(doc.get_cursor()), c.to_string())
        }

        doc.reset_cursor();
        doc.apply_operation(DeltaOperation::retain(i + 1))?;
        //error!("DONE retain_less_simple_test()");
    }
    Ok(())
}
