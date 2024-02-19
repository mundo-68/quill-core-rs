use dom::dom_element::DomElement;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Element, Node};

wasm_bindgen_test_configure!(run_in_browser);
pub static DIV: &str = "DIV";

fn create_structure(root: &DomElement) -> &DomElement {
    let n = DomElement::new(DIV);
    n.set_attribute("id", "1");
    root.append_child(n.node());

    let n = DomElement::new(DIV);
    n.set_attribute("id", "2");
    root.append_child(n.node());

    let n = DomElement::new(DIV);
    n.set_attribute("id", "3");
    root.append_child(n.node());

    root
}

#[wasm_bindgen_test]
fn append_child_test() {
    let doc = DomElement::new(DIV);
    doc.set_attribute("ID", "append_child");
    let root = create_structure(&doc);
    let expect = "<div id=\"1\"></div><div id=\"2\"></div><div id=\"3\"></div>";
    assert_eq!(root.element().inner_html(), expect);
}

#[wasm_bindgen_test]
fn insert_child_before_test() {
    let doc = DomElement::new(DIV);
    doc.set_attribute("ID", "insert_child_before_test");
    let root = create_structure(&doc);

    let c = root.get_child(2).unwrap();
    let v = c.has_type::<Element>();
    assert!(v);

    let n: &Element = c.dyn_ref::<Element>().unwrap();
    let xx = DomElement::new(DIV);
    xx.set_attribute("id", "4");

    root.insert_child_before(xx.node(), n);

    let expect = "<div id=\"1\"></div><div id=\"2\"></div><div id=\"4\"></div><div id=\"3\"></div>";
    assert_eq!(root.element().inner_html(), expect);
}

#[wasm_bindgen_test]
fn insert_child_test() {
    let doc = DomElement::new(DIV);
    doc.set_attribute("ID", "insert_child_test");
    let root = create_structure(&doc);

    let c = root.get_child(2).unwrap();
    let v = c.has_type::<Element>();
    assert!(v);

    let xx = DomElement::new(DIV);
    xx.set_attribute("id", "4");

    root.insert_child(2, xx.node());

    let expect = "<div id=\"1\"></div><div id=\"2\"></div><div id=\"4\"></div><div id=\"3\"></div>";
    assert_eq!(root.element().inner_html(), expect);
}

#[wasm_bindgen_test]
fn replace_child_test() {
    let doc = DomElement::new(DIV);
    doc.set_attribute("ID", "replace_child");
    let root = create_structure(&doc);

    let c = root.get_child(1).unwrap();
    assert!(c.has_type::<Element>());

    let xx = DomElement::new(DIV);
    xx.set_attribute("id", "5");

    root.replace_child(&c, xx.node());

    let c: Node = root.get_child(1).unwrap();
    let el: &Element = c.dyn_ref::<Element>().unwrap();
    assert_eq!(el.get_attribute("id").unwrap(), "5");

    let expect = "<div id=\"1\"></div><div id=\"5\"></div><div id=\"3\"></div>";
    assert_eq!(root.element().inner_html(), expect);
}

#[wasm_bindgen_test]
fn get_children_test() {
    let doc = DomElement::new(DIV);
    doc.set_attribute("ID", "get_children");
    let root = create_structure(&doc);

    let c: Node = root.get_child(1).unwrap();
    let v = c.has_type::<Element>();
    assert!(v);

    let n: &Element = c.dyn_ref::<Element>().unwrap();
    assert_eq!(n.get_attribute("id").unwrap(), "2");

    let i: u32 = root.get_children().length();
    assert_eq!(i, 3);
}

#[wasm_bindgen_test]
fn move_children_test() {
    let doc = DomElement::new(DIV);
    doc.set_attribute("ID", "move_children_test");
    let root = create_structure(&doc);

    let n = DomElement::new(DIV);
    n.set_attribute("id", "4");
    root.append_child(n.node());
    assert_eq!(root.get_children().length(), 4);

    let new_parent = DomElement::new(DIV);
    new_parent.set_attribute("id", "3.141592");
    root.move_children_to_new_parent(&new_parent);
    root.append_child(new_parent.node());

    assert_eq!(new_parent.get_children().length(), 4);
    assert_eq!(root.get_children().length(), 1);
    let expect = "<div id=\"3.141592\"><div id=\"1\"></div><div id=\"2\"></div><div id=\"3\"></div><div id=\"4\"></div></div>";
    assert_eq!(root.element().inner_html(), expect);
}

#[wasm_bindgen_test]
fn find_up_down_test() {
    let root = DomElement::new(DIV);
    root.set_attribute("ID", "find_up_down_test");

    //create a NESTED structure of DIF
    let n1 = DomElement::new(DIV);
    n1.set_attribute("id", "1");
    root.append_child(n1.node());

    let n2 = DomElement::new(DIV);
    n2.set_attribute("id", "2");
    n1.append_child(n2.node());

    let n3 = DomElement::new(DIV);
    n3.set_attribute("id", "3");
    n2.append_child(n3.node());

    let n4 = DomElement::new(DIV);
    n4.set_attribute("id", "4");
    n3.append_child(n4.node());

    //let html = dom_print::pretty_delimited_print(root.get_element().unwrap(), true);
    //error!( "{:?}", &html);

    let del = n1;
    assert_ne!(del.get_child(0), None);
    //let html = dom_print::pretty_print(&del, true);
    //error!( "{:?}", &html);
    //error!( "find_down() ---> {:?}" , del.find_down( DIV ) );

    let te1 = del.find_down("DIV").unwrap();
    let del2 = DomElement::from(te1);
    assert_eq!(del2.get_attribute("id").unwrap(), "2");

    //Finding up includes the current element
    let te2 = del2.find_up("DIV").unwrap();
    let del3 = DomElement::from(te2);
    assert_eq!(del3.get_attribute("id").unwrap(), "1");
}

#[wasm_bindgen_test]
fn move_to_new_parent_test() {
    let doc = DomElement::new(DIV);
    doc.set_attribute("ID", "move_to_new_parent_test");
    let root = create_structure(&doc);

    let new_parent = DomElement::new(DIV);
    new_parent.set_attribute("id", "3.141592");

    let c: Node = root.get_child(1).unwrap();
    let el: Element = c.dyn_into::<Element>().unwrap();
    let del = DomElement::from(el);

    let expect = "<div id=\"2\"></div>";
    assert_eq!(&del.element().outer_html(), expect);

    del.move_to_new_parent(1, &new_parent);

    let expect = "<div id=\"3.141592\"><div id=\"2\"></div></div>";
    assert_eq!(&new_parent.element().outer_html(), expect);

    let expect = "<div id=\"1\"></div><div id=\"3\"></div>";
    assert_eq!(root.element().inner_html(), expect);
}

#[wasm_bindgen_test]
fn add_remove_style_test() {
    let root = DomElement::new(DIV);
    root.set_attribute("ID", "add_style_test");

    let new_parent = DomElement::new(DIV);
    root.append_child(new_parent.node());

    //Add first
    DomElement::add_style(&new_parent, "font-size", "1.5em");
    let style = new_parent.get_attribute("style").unwrap();
    assert_eq!(style, "font-size:1.5em;");
    let expect = r##"<div style="font-size:1.5em;"></div>"##;
    assert_eq!(new_parent.element().outer_html(), expect);

    //Add second
    DomElement::add_style(&new_parent, "background-color", "red");
    let style = new_parent.get_attribute("style").unwrap();
    assert_eq!(style, "font-size:1.5em;background-color:red;");
    let expect = r##"<div style="font-size:1.5em;background-color:red;"></div>"##;
    assert_eq!(new_parent.element().outer_html(), expect);

    //Update first
    DomElement::add_style(&new_parent, "font-size", "2.5em");
    let style = new_parent.get_attribute("style").unwrap();
    assert_eq!(style, "font-size:2.5em;background-color:red;");
    let expect = r##"<div style="font-size:2.5em;background-color:red;"></div>"##;
    assert_eq!(new_parent.element().outer_html(), expect);

    //Add third
    DomElement::add_style(&new_parent, "color", "green");
    let style = new_parent.get_attribute("style").unwrap();
    assert_eq!(style, "font-size:2.5em;background-color:red;color:green;");
    let expect = r##"<div style="font-size:2.5em;background-color:red;color:green;"></div>"##;
    assert_eq!(new_parent.element().outer_html(), expect);

    //drop second
    //We check if only part of the string suffices background-color --> background-
    DomElement::remove_style(&new_parent, "background-");
    let style = new_parent.get_attribute("style").unwrap();
    assert_eq!(style, "font-size:2.5em;color:green;");
    let expect = r##"<div style="font-size:2.5em;color:green;"></div>"##;
    assert_eq!(new_parent.element().outer_html(), expect);
}
