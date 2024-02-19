// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::format_const::{
    FORMAT_BOLD, FORMAT_DELETED, FORMAT_INSERTED, FORMAT_ITALIC, FORMAT_MARKED, FORMAT_SMALL,
    FORMAT_STRIKE, FORMAT_SUB, FORMAT_SUP, FORMAT_UNDERLINE,
};
use crate::util::lookup::{AttributesLookup, Attributor};
use delta::attributes::Attributes;
use delta::types::attr_val::AttrVal;
use dom::dom_element::DomElement;
use log::debug;
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::{insert_before, remove_child};
use once_cell::sync::OnceCell;
/// text_format only handles Operational transform with signature:
///  -  `{ insert : String, Attributes[FORMATS] }`
/// Where the supported formats are described below. Other formats are ignored ...
///
/// We want to stick to the sequence order of formats as in the static array.
///
use std::sync::Arc;
use web_sys::Element;

// //will end up as element child; Example for paragraph <p> ... <format></format> ... </p>
pub static TEXT_FORMATS: OnceCell<AttributesLookup> = OnceCell::new();
pub fn initialise() {
    if let Some(_attr) = TEXT_FORMATS.get() {
        return;
    }
    let mut attr = AttributesLookup::new(8);
    attr.fill_one(FORMAT_BOLD, "strong");
    //"emphasize" => "em",
    attr.fill_one(FORMAT_ITALIC, "em");
    attr.fill_one(FORMAT_UNDERLINE, "U");
    attr.fill_one(FORMAT_STRIKE, "S");
    attr.fill_one(FORMAT_SUB, "SUB");
    attr.fill_one(FORMAT_SUP, "sup");
    attr.fill_one(FORMAT_DELETED, "DEL");
    attr.fill_one(FORMAT_INSERTED, "INS");
    attr.fill_one(FORMAT_MARKED, "MARK");
    attr.fill_one(FORMAT_SMALL, "SMALL");

    TEXT_FORMATS
        .set(attr)
        .expect("did you call TEXT_FORMATS::initialise() twice?");
}

#[inline]
pub(crate) fn has_text_fomat(attr: &Attributes) -> bool {
    let text_attr = Attributor::all_key(TEXT_FORMATS.get().unwrap());
    let applies = text_attr
        .filter(|&&key| attr.contains_key(key))
        .collect::<Vec<&&str>>();
    !applies.is_empty()
}

//When creating a new node, this method preserves the sequence of formats in the "chain"
//This is done by iterating over the keys array, which may be a bit slower, than just checking
//if some attribute is a member of the key set in the attribute map.
//FIXME: Removing an attribute and adding one later will "mess-up" the nice order of formats
pub(crate) fn apply_text_formats(
    doc_node: &Arc<DocumentNode>,
    attr: &Attributes,
) -> Arc<DocumentNode> {
    let mut dn = doc_node.clone();
    for (format, attr_val) in Attributor::selected(attr, TEXT_FORMATS.get().unwrap()) {
        if let AttrVal::Bool(b) = attr_val {
            if *b {
                dn = add_one_text_format(&dn, format);
            }
        }
        if &AttrVal::Null == attr_val {
            dn = remove_one_text_format(&dn, format);
        }
    }
    dn.set_formatter(&doc_node.get_formatter());
    dn.set_operation(doc_node.get_operation());
    dn.clone()
}

#[inline(always)]
fn add_one_text_format(doc_node: &Arc<DocumentNode>, format: &str) -> Arc<DocumentNode> {
    let parent_o = doc_node.get_parent();
    if parent_o.is_some() {
        //remove child from parent first, so that we can attach the element to the new parent below
        //we need the parent later, so we do not "borrow" parent_o
        remove_child(&doc_node.get_parent().unwrap(), doc_node);
    }
    let el_o = doc_node.get_dom_element();
    let dom_el = if let Some(el) = el_o {
        let format_element = el.find_down(format); //find excludes current
        if format_element.is_none() && el.node_name() != format {
            let dom_el = DomElement::new(format);
            dom_el.append_child(el.node());
            dom_el
        } else {
            //case: we already have this format applied --> no action required in that case
            return doc_node.clone();
        }
    } else {
        //We are a TextNode :-) --> first format.
        let text = doc_node.get_dom_text().unwrap();
        let dom_el = DomElement::new(format);
        dom_el.append_child(text.node());
        dom_el
    };

    let dd = DocumentNode::new_element(dom_el, doc_node.get_formatter().clone());
    let new_doc_node = Arc::new(dd);
    if let Some(parent) = parent_o {
        insert_before(&parent, doc_node, new_doc_node.clone());
    } else {
        //Coming from Compose() ?
        //debug!( "Applying an attribute to a doc_node which does not have a parent !!" );
    }
    new_doc_node
}

#[inline(always)]
fn remove_one_text_format(doc_node: &Arc<DocumentNode>, format: &str) -> Arc<DocumentNode> {
    let parent = doc_node.get_parent().unwrap();
    let p_el_o = doc_node.get_dom_element();
    if let Some(new_dom_el) = p_el_o {
        if new_dom_el.node_name() == format {
            //handling first format == doc_node.element
            let dd = DocumentNode::new_node(
                new_dom_el.get_child(0).unwrap(),
                doc_node.get_formatter().clone(),
            );
            let new_doc_node = Arc::new(dd);
            new_dom_el.remove_child_index(0);
            insert_before(&parent, doc_node, new_doc_node.clone());
            remove_child(&parent, doc_node);
            return new_doc_node;
        } else {
            //handling next format (grand) child of doc_node.element
            let f: Option<Element> = new_dom_el.find_down(format);
            match f {
                Some(el) => {
                    let rm_el = DomElement::from(el);
                    let parent_el = DomElement::from(rm_el.get_parent().unwrap());
                    let child_node = rm_el.get_child(0).unwrap();
                    parent_el.insert_child_before(&child_node, rm_el.node());
                    parent_el.remove_child(rm_el.node());
                }
                _ => {
                    //we do not have this format applied --> return parent "as is"
                    debug!("remove() --> format {} is not applied, skipping.", format);
                    return doc_node.clone();
                }
            }
        };
    }

    //we do not have this format applied --> we are probably a TEXT element
    doc_node.clone()
}

#[cfg(test)]
mod test {
    use super::*;
    use delta::types::attr_val::AttrVal;
    use delta::types::attr_val::AttrVal::Null;
    use dom::dom_text::DomText;
    use node_tree::dom_doc_tree_morph::*;
    use node_tree::format_trait::RootFormat;
    use op_transform::doc_root::DocumentRoot;
    use std::sync::Arc;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn apply_format_test() {
        super::initialise();

        let doc = DocumentRoot::new("apply_format_test");
        doc.append_to_body();

        let el = DomText::new("apply_format_test");
        let mut doc_node = Arc::new(DocumentNode::new_text(
            el,
            Arc::new(RootFormat::new("t_format")),
        ));

        append(doc.get_root(), doc_node.clone());
        assert_eq!(doc.get_root().child_count(), 1);

        let mut attr = Attributes::default();
        attr.insert("bold", true);
        doc_node = apply_text_formats(&doc_node, &attr);
        assert_eq!(doc.get_root().child_count(), 1);
        let expect = "<strong>apply_format_test</strong>";
        assert_eq!(doc.as_html_string(), expect);

        let mut attr = Attributes::default();
        attr.insert("superscript", true);
        doc_node = apply_text_formats(&doc_node, &attr);
        assert_eq!(doc.get_root().child_count(), 1);

        let mut attr = Attributes::default();
        attr.insert("italic", true);
        doc_node = apply_text_formats(&doc_node, &attr);
        assert_eq!(doc.get_root().child_count(), 1);
        let html = doc.as_html_string();
        assert!(html.contains("sup"));
        assert!(html.contains("strong"));
        assert!(html.contains("em"));

        let mut attr = Attributes::default();
        attr.insert("superscript", Null);
        doc_node = apply_text_formats(&doc_node, &attr);
        assert_eq!(doc.get_root().child_count(), 1);
        let html = doc.as_html_string();
        assert!(!html.contains("sup"));
        assert!(html.contains("strong"));
        assert!(html.contains("em"));

        let mut attr = Attributes::default();
        attr.insert("bold", Null);
        apply_text_formats(&doc_node, &attr);
        assert_eq!(doc.get_root().child_count(), 1);
        let html = doc.as_html_string();
        assert!(!html.contains("sup"));
        assert!(!html.contains("strong"));
        assert!(html.contains("em"));

        let mut attr = Attributes::default();
        attr.insert("bold", Null);
        apply_text_formats(&doc_node, &attr);
        assert_eq!(doc.get_root().child_count(), 1);
        doc.as_html_string();
        assert!(!html.contains("sup"));
        assert!(!html.contains("em"));
    }

    #[test]
    pub fn has_text_format_test() {
        initialise();
        let mut attr = Attributes::default();
        attr.insert("bold".to_string(), AttrVal::from(true));
        attr.insert("small".to_string(), AttrVal::from(true));
        attr.insert("some".to_string(), AttrVal::from("thing"));
        attr.insert("else".to_string(), AttrVal::from("to_do"));
        let v = has_text_fomat(&attr);
        assert_eq!(v, true);

        let mut attr = Attributes::default();
        attr.insert("some".to_string(), AttrVal::from("thing"));
        attr.insert("else".to_string(), AttrVal::from("to_do"));
        let v = has_text_fomat(&attr);
        assert_eq!(v, false);
    }
}
