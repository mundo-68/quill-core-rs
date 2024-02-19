use anyhow::Result;
use dom::constants::DOCUMENT;
use dom::dom_element::get_dom_element_by_id;
use op_transform::doc_root::DocumentRoot;
use wasm_bindgen_test::wasm_bindgen_test_configure;
use wasm_bindgen_test::*;
use web_sys::Element;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn new_test() -> Result<()> {
    let doc = DocumentRoot::new(&*"huppeldepup");
    assert_eq!(
        doc.get_root().get_dom_element().unwrap().node().node_type(),
        1 as u16
    ); //node.element
    assert_eq!(doc.get_root().get_dom_element().unwrap().node_name(), "DIV");
    assert_eq!(
        doc.get_container_element().get_attribute("id").unwrap(),
        "huppeldepup"
    );

    let expect = r##"<div class="ql-editor"></div>"##;
    assert_eq!(doc.as_outer_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn bind_to_test() -> Result<()> {
    let n: Element = DOCUMENT.with(|d| d.create_element("DIV").unwrap());
    n.set_attribute("id", "kind")
        .expect("Document:bind_to_test()");

    let doc = DocumentRoot::new(&*"doc-root");
    doc.bind_to(n.as_ref());

    let expect =
        r##"<div id="doc-root" class="ql-container ql-snow"><div class="ql-editor"></div></div>"##;
    assert_eq!(doc.get_container_element().element().outer_html(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn bind_to_body_test() -> Result<()> {
    let doc = DocumentRoot::new(&*"body-root");

    //We do not expect to find, so we check for none on the element
    let el = get_dom_element_by_id(&*"body-root");
    assert_eq!(el, None);

    doc.append_to_body();

    let el2 = get_dom_element_by_id(&*"body-root").unwrap();
    assert_eq!(el2.get_attribute("id").unwrap(), &*"body-root");
    Ok(())
}
