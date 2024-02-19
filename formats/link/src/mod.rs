// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use anyhow::Result;
use core_formats::util::lookup::{AttributesLookup, Attributor};
use core_formats::util::node_morph::{
    delete_node, delete_text, merge_block_node, split_block_before_child, split_text_node,
    try_3_way_merge_block, try_3_way_merge_text,
};
use core_formats::TEXT_FORMAT;
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use dom::dom_element::DomElement;
use node_tree::cursor::{Cursor, CursorLocation};
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::{append, unlink};
use node_tree::format_trait::FormatTait;
use node_tree::tree_traverse::{next_sibling, prev_sibling};
use once_cell::sync::OnceCell;
use std::sync::Arc;

//Fixme: Delete last text in link should delete link ...

pub static NAME_LINK: &'static str = "link"; //registry label

static LINK_ATTR: &'static str = "link"; //marker attribute to recognize this format
pub static LINK_TAG: &'static str = "A"; //HTML tag

static ATTRIBUTES: OnceCell<AttributesLookup> = OnceCell::new();
pub(crate) fn initialise() {
    if let Some(_attr) = ATTRIBUTES.get() {
        return;
    }
    let mut attr = AttributesLookup::new(1);
    attr.fill_one(LINK_ATTR, "href");
    ATTRIBUTES
        .set(attr)
        .expect("did you call link::initialise() twice?");
}

/// # LinkFormat
///
/// LINK is an element which may have embedded markup, all inside a paragraph `<P>` block or other
/// vertical spacing block. So we can **NOT** design the operation format as a block format, like:
/// ```json
///      { op:"Google" }
///      { op:"\n",  attributes: { link: 'https://www.google.com' }}
///```
///
/// What did we design ...?
///
/// Insert a simple link:
/// ```json
///        { insert: "sweet", attributes: { link: 'https://www.google.com' } }
/// ```
/// results in:
/// ```html
///        <A href="https://www.google.com">
///           sweet
///        </A>
///```
///
/// Insert a formatted link:
/// ```json
///    {insert: "hello "}
///    {attributes: {link: "http:www.google.com"}, insert: "sw"}
///    {attributes: {bold: true, link: "http:www.google.com"}, insert: "e"}
///    {attributes: {link: "http:www.google.com"}, insert: "et"}
///    {insert: " world↵"}
///```
/// results in:
/// ```html
///   <P>Hello
///       <A href="http:www.google.com">
///         sw
///         <strong>e</strong>
///         et
///      </A>
///      World
///   </P>
///```
/// This leads to a DocumentNode structure as follows:
///  -
///
/// Sequence of handling:
/// 1) Create `<a>` element with doc node + format = LINK
/// 2) Create `<text>` node with doc node + format = LINK --> add to parent `<a>`
/// 3) Append `<a>` parent doc-node
/// 4) check left & right child for merger
///
/// Note: Splitting a leaf should split text and `<a>`
/// After a change we should check if we need to merge again
///
/// CURSOR<br>
/// The `<A>` node has length 0 and will be skipped by the operations: `insert`/`retain`/`delete`
///
/// This makes the following cursor locations "equivalent": <br>
///   `<A>text[*]</A>, and <A>text</A>[*]` <br>
/// and also <br>
///   `<A>[*]text</A>, and [*]<A>text</A>`
///
/// This link module shall !! make sure it can handle this ambiguity. The rest of the code must
/// be able to ignore this, and let the handling be done in the render module
///
/// Another caveat is that the text nodes may be formatted. Hence the child of some text
/// may be a `<EM>` or other formatting `html node`. So look for the right parent!
///
pub struct LinkFormat {}

impl LinkFormat {
    pub fn new() -> Self {
        initialise();
        LinkFormat {}
    }
}

impl FormatTait for LinkFormat {
    /// returns the `root` `DocumentNode` to the created link structure
    fn create(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let attr = operation.get_attributes().clone();
        let text_node = TEXT_FORMAT.create(operation, formatter.clone())?;

        let link_element = DomElement::new(LINK_TAG);
        for (k, v) in Attributor::selected(&attr, ATTRIBUTES.get().unwrap()) {
            link_element.set_attribute(&k, &v.str_val()?);
        }

        let op = DeltaOperation::insert_attr("", attr.clone());

        let dn = DocumentNode::new_element(link_element, formatter);
        dn.set_operation(op);
        let link_node = Arc::new(dn);
        append(&link_node, text_node);
        return Ok(link_node);
    }

    //Transforms a <P> text element in to some other block element
    fn format_name(&self) -> &'static str {
        return NAME_LINK;
    }

    fn is_text_format(&self) -> bool {
        true
    }

    fn block_remove_attr(&self) -> Attributes {
        panic!("Hey you called LinkFormat::block_remove_attr() on the TextFormatter format-trait implementation.");
    }

    fn applies(&self, delta: &DeltaOperation) -> Result<bool> {
        if let Some(_l) = delta.get_attributes().get(LINK_ATTR) {
            return Ok(true);
        }
        return Ok(false);
    }

    fn apply_line_attributes(
        &self,
        doc_node: &Arc<DocumentNode>,
        attr: &Attributes,
        _formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let dom_el = doc_node.get_dom_element().unwrap();
        for (k, v) in Attributor::selected(attr, ATTRIBUTES.get().unwrap()) {
            dom_el.set_attribute(k, &v.str_val()?)
        }
        if doc_node.get_operation().op_len() > 0 {
            //now do parent with <A> tag too
            let dn = doc_node.get_parent().unwrap();
            let dom_el = dn.get_dom_element().unwrap();
            for (k, v) in Attributor::selected(attr, ATTRIBUTES.get().unwrap()) {
                dom_el.set_attribute(k, &v.str_val()?)
            }
        }
        Ok(doc_node.clone())
    }

    fn drop_line_attributes(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        let dom_el = doc_node.get_dom_element().unwrap();
        let lookup = ATTRIBUTES.get().unwrap();
        for key in Attributor::all_key(lookup) {
            dom_el.remove_attribute(&key);
        }
        if doc_node.get_operation().op_len() > 0 {
            //now do parent with <A> tag too
            let dn = doc_node.get_parent().unwrap();
            let dom_el = dn.get_dom_element().unwrap();
            for key in Attributor::all_key(lookup) {
                dom_el.remove_attribute(&key);
            }
        }
        Ok(doc_node.clone())
    }

    fn clone_doc_node(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        return if doc_node.get_doc_dom_node().get_node_name() == LINK_TAG {
            let dn = self.create(
                doc_node.get_operation().clone(),
                doc_node.get_formatter().clone(),
            )?;
            //We return an EMPTY <A> node if we are a link ...
            if dn.child_count() > 0 {
                let child = dn.get_child(0).unwrap();
                if child.op_len() == 0 {
                    unlink(&dn, &child);
                }
            }
            Ok(dn)
        } else {
            self.create(doc_node.get_operation(), doc_node.get_formatter())
        };
    }

    fn split_leaf(&self, cursor: &Cursor) -> Result<()> {
        assert_ne!(
            cursor.get_doc_node().get_doc_dom_node().get_node_name(),
            LINK_TAG
        );
        link_split_leaf(cursor)?;
        Ok(())
    }

    fn is_same_format(&self, left: &Arc<DocumentNode>, right: &Arc<DocumentNode>) -> bool {
        if let Some(left) = left.get_operation().get_attributes().get(LINK_ATTR) {
            if let Some(right) = right.get_operation().get_attributes().get(LINK_ATTR) {
                if left.eq(right) {
                    return true;
                }
            }
        }
        return false;
    }

    fn block_transform(
        &self,
        _cursor: &Cursor,
        _block_node: &Arc<DocumentNode>,
        _delta: DeltaOperation,
        _new_transducer: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        panic!("LinkFormat::block_transform() - Error.");
    }

    fn un_block_transform(
        &self,
        _cursor: &Cursor,
        _block_node: &Arc<DocumentNode>,
    ) -> Result<Arc<DocumentNode>> {
        panic!("LinkFormat::un_block_transform() - Error.");
    }

    fn delete_leaf_segment(
        &self,
        doc_node: &Arc<DocumentNode>,
        at: usize,
        length: usize,
    ) -> Result<()> {
        //assert_eq!(doc_node.get_doc_dom_node().get_node_name(), LINK_TAG );
        //assert_eq!(doc_node.child_count(),1);
        //text::delete_text(&doc_node.get_children().get(0).unwrap().clone(), at, length);
        assert_ne!(doc_node.get_doc_dom_node().get_node_name(), LINK_TAG);
        assert_eq!(
            doc_node
                .get_parent()
                .unwrap()
                .get_doc_dom_node()
                .get_node_name(),
            LINK_TAG
        );
        delete_text(&doc_node, at, length)?;
        Ok(())
    }

    fn delete_node(&self, doc_node: &Arc<DocumentNode>) {
        if doc_node.get_doc_dom_node().get_node_name() != LINK_TAG {
            let parent = doc_node.get_parent().unwrap();
            //let xxx_p_parent = parent.get_parent().unwrap();
            assert_eq!(parent.get_doc_dom_node().get_node_name(), LINK_TAG);
            delete_node(doc_node);
            if parent.child_count() == 0 {
                delete_node(&parent);
            }
        } else {
            assert_eq!(doc_node.get_doc_dom_node().get_node_name(), LINK_TAG);
            delete_node(doc_node);
        }
    }

    /// We split only if resulting link node is non empty
    fn isolate(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        if doc_node.get_doc_dom_node().get_node_name() != LINK_TAG {
            let mut ul_node = doc_node.get_parent().unwrap();
            if let Some(nxt) = next_sibling(doc_node) {
                split_block_before_child(&ul_node, &nxt)?;
            }
            if let Some(_prv) = prev_sibling(doc_node) {
                ul_node = split_block_before_child(&ul_node, &doc_node)?;
            }
            return Ok(ul_node);
        }
        return Ok(doc_node.clone());
    }

    fn try_merge(&self, cursor: &Cursor, _block_node: &Arc<DocumentNode>) -> Result<()> {
        try_merge_link(cursor)?;
        Ok(())
    }
}

/// After splitting up the link node, the cursor points to the "LINK_TAG" doc node.
/// After the merging we should again point to the link tag node remaining??
fn try_merge_link(cursor: &Cursor) -> Result<()> {
    // let html = html_string(get_root(&cursor.get_doc_node()).get_html_node());
    // error!("try_merge_link: before {}", html);
    // error!("try_merge_link: before {}", &cursor);

    //Find the block that was probably changed
    let (after, doc_node) = match cursor.get_location() {
        CursorLocation::Before(doc_node) => {
            if let Some(prv) = prev_sibling(&doc_node) {
                (true, prv)
            } else {
                (false, doc_node)
            }
        }
        CursorLocation::After(doc_node) => (true, doc_node),
        CursorLocation::At(_doc_node, _i) => return Ok(()),
        _ => {
            return Ok(());
        }
    };

    //Find the link node tag
    let link_node = if doc_node.get_doc_dom_node().get_node_name() == LINK_TAG {
        if after {
            cursor.set_after(
                &doc_node
                    .get_children()
                    .get(doc_node.child_count() - 1)
                    .unwrap()
                    .clone(),
            );
        } else {
            cursor.set_before(&doc_node.get_children().get(0).unwrap().clone());
        }
        doc_node
    } else {
        if after {
            cursor.set_after(&doc_node);
        } else {
            cursor.set_before(&doc_node);
        }
        doc_node.get_parent().unwrap()
    };

    //error!("try_merge_link: merging {}", &link_node);

    //Might be needed in case of insert() action. See explanation in the op_transform module
    if let Some(prv) = prev_sibling(&link_node) {
        if let Some(prv_prv) = prev_sibling(&prv) {
            merge_block_node(&prv_prv, &prv)?;
        }
    }
    //Might be needed in case of delete() action. See explanation in the op_transform module
    if let Some(nxt) = next_sibling(&link_node) {
        if let Some(nxt_nxt) = next_sibling(&nxt) {
            merge_block_node(&nxt, &nxt_nxt)?;
        }
    }
    try_3_way_merge_block(&link_node)?;
    try_3_way_merge_text(&cursor)?;
    Ok(())

    // let html = html_string(get_root(&cursor.get_doc_node()).get_html_node());
    // error!("try_merge_link: after {}", html);
    // error!("try_merge_link: after {}", &cursor);
}

/// Input: pointer to a text node IN the link; returns the pointer to the <a> node
/// split @3: <a href="">hello</a> --> <a href="">hel</a>###<a href="">lo</a>
/// With ### the location of the cursor splitting left and right node
/// There is no such thing as an empty link <a> node. A link ALWAYS has some text.
/// So the cursor can never point to a document node which contains an empty link.
/// Post condition: Cursor position is BEFORE the right hand side created node
fn link_split_leaf(cursor: &Cursor) -> Result<()> {
    // let html = html_string(get_root(&cursor.get_doc_node()).get_html_node());
    // error!("link_split_leaf: before {}", html);
    // error!("link_split_leaf: before {}", &cursor);

    assert_eq!(
        cursor.get_doc_node().get_formatter().format_name(),
        NAME_LINK
    );
    assert_ne!(
        cursor.get_doc_node().get_doc_dom_node().get_node_name(),
        LINK_TAG
    );

    match cursor.get_location() {
        CursorLocation::At(doc_node, index) => {
            let right = split_text_node(&doc_node, index)?;
            cursor.set_before(&right);
            //error!("link_split_leaf: **AT** {}", &cursor);
            link_split_leaf(cursor)?; //recurse using the BEFORE[doc_node] matching case
        }
        CursorLocation::Before(doc_node) => {
            if let Some(prev) = prev_sibling(&doc_node) {
                //only split if there is a previous node otherwise we get empty <A> blocks
                let left_parent = doc_node.get_parent().unwrap();
                split_block_before_child(&left_parent, &doc_node)?;
                if let Some(_prv) = prev_sibling(&prev) {
                    //only split if there is a prev node otherwise we get empty <A> blocks
                    //--> hier gaat het fout ... prev is geen kind van link ??
                    split_block_before_child(&left_parent, &prev)?;
                }
            }
            cursor.set_before(&doc_node.get_parent().unwrap());
        }
        CursorLocation::After(doc_node) => {
            if let Some(next) = next_sibling(&doc_node) {
                //only split if there is a previous node otherwise we get empty <A> blocks
                split_block_before_child(&doc_node.get_parent().unwrap(), &next)?;
            }
            if let Some(_prev) = prev_sibling(&doc_node) {
                //only split if there is a prev node otherwise we get empty <A> blocks
                split_block_before_child(&doc_node.get_parent().unwrap(), &doc_node)?;
            }
            cursor.set_after(&doc_node.get_parent().unwrap());
        }
        CursorLocation::None => {}
    }
    Ok(())

    // let html = html_string(get_root(&cursor.get_doc_node()).get_html_node());
    // error!("link_split_leaf: after {}", html);
    // error!("link_split_leaf: after {}", &cursor);
}
