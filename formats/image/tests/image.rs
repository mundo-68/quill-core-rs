use anyhow::Result;
use core_formats::format_const::{NAME_P_BLOCK, NAME_TEXT};
use core_formats::{P_FORMAT, TEXT_FORMAT};
use delta::attributes::Attributes;
use delta::delta::Delta;
use delta::operations::OpsMap;
use image::{ImageFormat, NAME_IMAGE};
use node_tree::cursor::Cursor;
use node_tree::format_trait::FormatTait;
use node_tree::tree_traverse::last_leaf_node;
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
        r.register_line_fmt(NAME_IMAGE, Arc::new(ImageFormat::new()))
            .unwrap();
        r.register_line_fmt(NAME_TEXT, TEXT_FORMAT.deref().clone())
            .unwrap();
        Mutex::new(1)
    });
}

fn create_test_img(doc: &mut DocumentRoot) -> Result<()> {
    let mut delta = Delta::default();

    let mut attr = Attributes::default();
    attr.insert("alt", "alt-text");
    //these have to be strings
    attr.insert("height", "60");
    //attr.insert("width", "50");

    let mut img = OpsMap::default();
    img.insert(NAME_IMAGE, "image-source.png");

    delta.insert_attr(img, attr);

    doc.open()?;
    doc.apply_delta(delta)?;

    let img = last_leaf_node(&doc.get_root()).unwrap();
    //error!( "print img {}",img);
    assert_eq!(img.op_len(), 1); //--> 1x "\n" character + image
    Ok(())
}

#[wasm_bindgen_test]
fn create_image_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("create_image_test");
    doc.append_to_body();
    create_test_img(&mut doc)?;

    let p = doc.get_root().get_children().get(0).unwrap().clone();
    let img = p.get_children().get(0).unwrap().clone();
    let len = img.op_len();
    assert_eq!(len, 1);

    // let expect =
    //     r##"<p><img img="image-source.png" alt="alt-text" width="50" height="60"></p>"##;
    let expect = r##"<p><img img="image-source.png" alt="alt-text" height="60"></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn drop_attributes_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("create_image_test");
    doc.append_to_body();
    create_test_img(&mut doc)?;

    let p = doc.get_root().get_children().get(0).unwrap().clone();
    let img = p.get_children().get(0).unwrap().clone();
    let len = img.op_len();
    assert_eq!(len, 1);

    let root = doc.get_root();
    let p = root.get_children().get(0).unwrap().clone();
    let img = p.get_children().get(0).unwrap().clone();

    let img = ImageFormat::new().drop_line_attributes(&img)?;

    let expect = r##"<p><img img="image-source.png"></p>"##;
    assert_eq!(doc.as_html_string(), expect);

    let mut attr = Attributes::default();
    attr.insert("width", "50");
    let _img = ImageFormat::new().apply_line_attributes(&img, &attr, img.get_formatter().clone());
    let expect = r##"<p><img img="image-source.png" width="50"></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn image_delete_test() -> Result<()> {
    init_test_registry();
    let mut doc = DocumentRoot::new("image_delete_test");
    doc.append_to_body();

    create_test_img(&mut doc)?;
    doc.cursor_to_start();

    let p = doc.get_root().get_children().get(0).unwrap().clone();
    let img = p.get_children().get(0).unwrap().clone();

    let test_cursor = Cursor::new();
    test_cursor.set_before(&img);

    let mut delta = Delta::default();
    delta.delete(1);

    doc.reset_cursor();
    doc.apply_delta(delta)?;

    let expect = r##"<p><br></p>"##;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}
