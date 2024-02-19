use dom::dom_element::DomElement;
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::{append, insert_at_index, remove_child_index, unlink};
use node_tree::format_trait::RootFormat;
use op_transform::doc_root::DocumentRoot;
use std::ptr;
use std::sync::Arc;
use wasm_bindgen_test::*;

// Support function to create a tests document
fn create_simple_doc(doc: &DocumentRoot) {
    let el1 = DomElement::new("DIV");
    el1.set_attribute("id", "0");
    let doc_node1 = DocumentNode::new_element(el1, Arc::new(RootFormat::new("dom_doc_tree_morph")));
    append(doc.get_root(), Arc::new(doc_node1));

    let el2 = DomElement::new("DIV");
    el2.set_attribute("id", "1");
    let doc_node2 = DocumentNode::new_element(el2, Arc::new(RootFormat::new("dom_doc_tree_morph")));
    append(doc.get_root(), Arc::new(doc_node2));

    let el3 = DomElement::new("DIV");
    el3.set_attribute("id", "2");
    let doc_node3 = DocumentNode::new_element(el3, Arc::new(RootFormat::new("dom_doc_tree_morph")));
    append(doc.get_root(), Arc::new(doc_node3));

    let expect = "<DIV id=\"0\"></DIV><DIV id=\"1\"></DIV><DIV id=\"2\"></DIV>";
    assert_eq!(doc.as_html_string(), expect);
}

#[wasm_bindgen_test]
fn get_parent_test() {
    let doc = DocumentRoot::new(&*"get_parent");

    create_simple_doc(&doc);
    let doc_node = doc.get_root().get_child(1).unwrap();
    assert!(ptr::eq(
        doc.get_root().as_ref(),
        doc_node.get_parent().unwrap().as_ref()
    ));
}

#[wasm_bindgen_test]
fn unlink_test() {
    let doc = DocumentRoot::new(&*"unlink_test");

    create_simple_doc(&doc);
    let doc_node = doc.get_root().get_child(1).unwrap();
    unlink(doc.get_root(), &doc_node);
    let expect = "<DIV id=\"0\"></DIV><DIV id=\"2\"></DIV>";
    assert_eq!(doc.as_html_string(), expect);
}

#[wasm_bindgen_test]
fn append_test() {
    let doc = DocumentRoot::new(&*"append_test");

    let element = DomElement::new("DIV");
    element.set_attribute("id", "kind");
    let doc_node =
        DocumentNode::new_element(element, Arc::new(RootFormat::new("dom_doc_tree_morph")));
    append(doc.get_root(), Arc::new(doc_node));
    let expect = "<DIV id=\"kind\"></DIV>";
    assert_eq!(doc.as_html_string(), expect);
}

#[wasm_bindgen_test]
fn child_index_test() {
    let doc = DocumentRoot::new(&*"child_index_test");

    create_simple_doc(&doc);

    let doc_node = doc.get_root().get_child(0).unwrap();
    assert_eq!(
        doc_node
            .get_dom_element()
            .unwrap()
            .get_attribute("id")
            .unwrap(),
        "0"
    );
    let idx = doc.get_root().get_child_index(&doc_node).unwrap();
    assert_eq!(idx, 0);

    let doc_node = doc.get_root().get_child(1).unwrap();
    assert_eq!(
        doc_node
            .get_dom_element()
            .unwrap()
            .get_attribute("id")
            .unwrap(),
        "1"
    );
    let idx = doc.get_root().get_child_index(&doc_node).unwrap();
    assert_eq!(idx, 1);

    let doc_node = doc.get_root().get_child(2).unwrap();
    assert_eq!(
        doc_node
            .get_dom_element()
            .unwrap()
            .get_attribute("id")
            .unwrap(),
        "2"
    );
    let idx = doc.get_root().get_child_index(&doc_node).unwrap();
    assert_eq!(idx, 2);
}

#[wasm_bindgen_test]
fn child_remove_index_test() {
    let doc = DocumentRoot::new(&*"child_remove_index_test-0");

    create_simple_doc(&doc);
    remove_child_index(&doc.get_root(), 1);
    let expect = "<DIV id=\"0\"></DIV><DIV id=\"2\"></DIV>";
    assert_eq!(doc.as_html_string(), expect);

    let doc1 = DocumentRoot::new(&*"child_remove_index_test-1");
    create_simple_doc(&doc1);
    remove_child_index(&doc1.get_root(), 0);
    let expect = "<DIV id=\"1\"></DIV><DIV id=\"2\"></DIV>";
    assert_eq!(doc.as_html_string(), expect);

    let doc2 = DocumentRoot::new(&*"child_remove_index_test-2");
    create_simple_doc(&doc2);
    remove_child_index(&doc2.get_root(), 2);
    let expect = "<DIV id=\"0\"></DIV><DIV id=\"1\"></DIV>";
    assert_eq!(doc.as_html_string(), expect);
}

#[wasm_bindgen_test]
fn child_insert_test() {
    let element = DomElement::new("DIV");
    element.set_attribute("id", "5");
    let doc_node = Arc::new(DocumentNode::new_element(
        element,
        Arc::new(RootFormat::new("dom_doc_tree_morph")),
    ));

    let doc = DocumentRoot::new(&*"child_insert_test-0");
    create_simple_doc(&doc);
    insert_at_index(&doc.get_root(), 0, doc_node.clone());
    let expect = "<DIV id=\"5\"></DIV><DIV id=\"0\"></DIV><DIV id=\"1\"></DIV><DIV id=\"2\"></DIV>";
    assert_eq!(doc.as_html_string(), expect);

    let doc1 = DocumentRoot::new(&*"child_insert_test-1");
    create_simple_doc(&doc1);
    insert_at_index(&doc1.get_root(), 1, doc_node.clone());
    let expect = "<DIV id=\"0\"></DIV><DIV id=\"5\"></DIV><DIV id=\"1\"></DIV><DIV id=\"2\"></DIV>";
    assert_eq!(doc.as_html_string(), expect);

    let doc2 = DocumentRoot::new(&*"child_insert_test-2");
    create_simple_doc(&doc2);
    insert_at_index(&doc2.get_root(), 3, doc_node.clone()); //inserts before, so this should become "append"
    let expect = "<DIV id=\"0\"></DIV><DIV id=\"1\"></DIV><DIV id=\"2\"></DIV><DIV id=\"5\"></DIV>";
    assert_eq!(doc.as_html_string(), expect);
}
