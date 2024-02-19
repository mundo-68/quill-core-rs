use anyhow::Result;
use dom::dom_element::DomElement;
use dom::dom_text::{find_dom_text, DomText};
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::append;
use node_tree::format_trait::RootFormat;
use std::sync::Arc;
use wasm_bindgen_test::wasm_bindgen_test_configure;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);
pub static DIV: &'static str = "DIV";
const DOC_ROOT_FORMAT: &str = "DOC_ROOT_FORMAT";

#[wasm_bindgen_test]
fn create() -> Result<()> {
    let el = DomElement::new(DIV);
    el.set_attribute("ID", "text-create");
    let root = Arc::new(DocumentNode::new_element(
        el,
        Arc::new(RootFormat::new(DOC_ROOT_FORMAT)),
    ));

    let t = DomText::new("hello WORLD");
    let doc_node = DocumentNode::new_text(
        t,
        Arc::new(RootFormat {
            name: "DOM_TEXT_TEST",
        }),
    );
    let doc_node_p = Arc::new(doc_node);
    append(&root, doc_node_p.clone());
    assert_eq!(
        root.get_dom_element().unwrap().element().inner_html(),
        "hello WORLD"
    );

    let l = doc_node_p.get_dom_text().unwrap().text_length();
    assert_eq!(l, 11);
    Ok(())
}

#[wasm_bindgen_test]
fn append_test() {
    let el = DomElement::new(DIV);
    el.set_attribute("ID", "text-append");
    let root = Arc::new(DocumentNode::new_element(
        el,
        Arc::new(RootFormat::new(DOC_ROOT_FORMAT)),
    ));

    let t = DomText::new("hello");
    let doc_node = DocumentNode::new_text(
        t,
        Arc::new(RootFormat {
            name: "DOM_TEXT_TEST",
        }),
    );
    append(&root, Arc::new(doc_node));

    let children = root.get_children();
    let tn = children.get(0).unwrap();
    let txt = tn.get_dom_text().unwrap();
    txt.append_text(" WORLD");

    assert_eq!(
        root.get_dom_element().unwrap().element().inner_html(),
        "hello WORLD"
    );
}

#[wasm_bindgen_test]
fn insert() {
    let el = DomElement::new(DIV);
    el.set_attribute("ID", "text-insert");
    let root = Arc::new(DocumentNode::new_element(
        el,
        Arc::new(RootFormat::new(DOC_ROOT_FORMAT)),
    ));

    let t = DomText::new("helloD");
    let doc_node = DocumentNode::new_text(
        t,
        Arc::new(RootFormat {
            name: "DOM_TEXT_TEST",
        }),
    );
    append(&root, Arc::new(doc_node));

    let children = root.get_children();
    let tn = children.get(0).unwrap();
    let txt = tn.get_dom_text().unwrap();
    txt.insert_text(5, " WORL");

    assert_eq!(
        root.get_dom_element().unwrap().element().inner_html(),
        "hello WORLD"
    );
}

#[wasm_bindgen_test]
fn delete() {
    let el = DomElement::new(DIV);
    el.set_attribute("ID", "text-delete");
    let root = Arc::new(DocumentNode::new_element(
        el,
        Arc::new(RootFormat::new(DOC_ROOT_FORMAT)),
    ));

    let t = DomText::new("hello WORLD WORLD");
    let doc_node = DocumentNode::new_text(
        t,
        Arc::new(RootFormat {
            name: "DOM_TEXT_TEST",
        }),
    );
    append(&root, Arc::new(doc_node));

    let children = root.get_children();
    let tn = children.get(0).unwrap();
    let txt = tn.get_dom_text().unwrap();
    txt.delete_text(10, 6);

    //test_match_with_parent(root.get_dom_element().unwrap().element(), "<DIV id=\"text-delete\">hello WORLD</DIV>");
    assert_eq!(
        root.get_dom_element().unwrap().element().inner_html(),
        "hello WORLD"
    );
}

#[wasm_bindgen_test]
fn get_text_test() {
    let el = DomElement::new(DIV);
    el.set_attribute("ID", "text-get");
    let root = Arc::new(DocumentNode::new_element(
        el,
        Arc::new(RootFormat::new(DOC_ROOT_FORMAT)),
    ));

    let t = DomText::new("hello WORLD");
    let doc_node = DocumentNode::new_text(
        t,
        Arc::new(RootFormat {
            name: "DOM_TEXT_TEST",
        }),
    );
    append(&root, Arc::new(doc_node));

    let children = root.get_children();
    let tn = children.get(0).unwrap();
    let txt = tn.get_dom_text().unwrap();
    let value = txt.get_text();

    assert_eq!(
        root.get_dom_element().unwrap().element().inner_html(),
        "hello WORLD"
    );
    assert_eq!(value, "hello WORLD".to_string());
}

#[wasm_bindgen_test]
fn find_text_node_test() {
    let el = DomElement::new(DIV);
    el.set_attribute("ID", "find_text_node_test");
    let root = Arc::new(DocumentNode::new_element(
        el,
        Arc::new(RootFormat::new(DOC_ROOT_FORMAT)),
    ));

    let t = DomText::new("hello WORLD");
    let doc_node = DocumentNode::new_text(
        t,
        Arc::new(RootFormat {
            name: "DOM_TEXT_TEST",
        }),
    );
    append(&root, Arc::new(doc_node));

    let children = root.get_children();
    let tn = children.get(0).unwrap();

    let text = find_dom_text(tn.get_html_node()).unwrap();

    assert_eq!(text.get_text(), "hello WORLD".to_string());

    text.delete_text(2, 7);
    assert_eq!(text.get_text(), "heLD".to_string());

    let text2 = find_dom_text(tn.get_html_node()).unwrap();
    assert_eq!(text2.get_text(), "heLD".to_string());
}

#[wasm_bindgen_test]
fn find_text_node_from_element_test() {
    let el = DomElement::new(DIV);
    el.set_attribute("ID", "find_text_node_from_element_test-create");
    let root = Arc::new(DocumentNode::new_element(
        el,
        Arc::new(RootFormat::new(DOC_ROOT_FORMAT)),
    ));

    let t = DomText::new("hello WORLD");
    let el = DomElement::new(DIV);
    el.append_child(t.node());

    let doc_node = DocumentNode::new_element(
        el,
        Arc::new(RootFormat {
            name: "DOM_TEXT_TEST",
        }),
    );
    append(&root, Arc::new(doc_node));

    let children = root.get_children();
    let test_node = children.get(0).unwrap();

    let text = find_dom_text(test_node.get_html_node()).unwrap();

    assert_eq!(text.get_text(), "hello WORLD".to_string());

    text.delete_text(2, 7);
    assert_eq!(text.get_text(), "heLD".to_string());

    let text2 = find_dom_text(test_node.get_html_node()).unwrap();
    assert_eq!(text2.get_text(), "heLD".to_string());
}
