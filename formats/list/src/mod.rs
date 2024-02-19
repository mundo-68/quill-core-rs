// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::list_const::{LIST_ATTR_KEY, LIST_BULLET, LIST_ORDERED};
use anyhow::Result;
use core_formats::util::block::{apply_attributes, drop_attributes};
use core_formats::util::block_format;
use core_formats::util::node_morph::{delete_node, merge_block_node, split_block_before_child};
use core_formats::P_FORMAT;
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use delta::types::attr_val::AttrVal;
use delta::types::attr_val::AttrVal::Null;
use dom::dom_element::DomElement;
use node_tree::cursor::Cursor;
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::{append, insert_at_index, insert_before, unlink};
use node_tree::format_trait::FormatTait;
use node_tree::tree_traverse::{next_sibling, prev_sibling};
use std::sync::Arc;

pub mod list_const;

pub static NAME_UL_BLOCK: &str = "UL_BLOCK";
pub static NAME_OL_BLOCK: &str = "OL_BLOCK";

static UL_TAG: &str = "UL";
static OL_TAG: &str = "OL";
static LI_TAG: &str = "LI";

/// # ListBlock
///
/// # UNSORTED LIST
///
/// ```html
/// <ul style="list-style-type:disc;">
///    <li>Coffee</li>
///    <li>Tea</li>
///    <li>Milk</li>
///  </ul>
/// ```
///
/// The CSS list-style-type property is used to define the style of the list item marker.
/// It can have one of the following values:
///
/// Value  |	Description:
/// - disc	| Sets the list item marker to a bullet (default)
/// - circle |	Sets the list item marker to a circle
/// - square |	Sets the list item marker to a square
/// - none	| The list items will not be marked
///
///
/// # ORDERED LIST
///
/// ```html
///<ol type="1">
///   <li>Coffee</li>
///   <li>Tea</li>
///   <li>Milk</li>
/// </ol>
/// ```
///
/// Ordered HTML List - The Type Attribute<br>
/// The type attribute of the `<OL>` tag, defines the type of the list item marker:
///
/// Type  |	Description:
/// - type="1"	The list items will be numbered with numbers (default)
/// - type="A"	The list items will be numbered with uppercase letters
/// - type="a"	The list items will be numbered with lowercase letters
/// - type="I"	The list items will be numbered with uppercase roman numbers
/// - type="i"	The list items will be numbered with lowercase roman numbers
///
/// Design:
///
/// We create a Delta with content of 2 lines:
/// ```json
///  {"insert": "This text is a normal first line \n list line 1"},  // content
///  {"insert": "\n", "attributes":  {"list":"bullet"}},             // block format
///  {"insert": " list line 2"},                                     // content
///  {"insert": "\n", "attributes":  {"list":"bullet"}}              // block format
/// ```
///
/// This gets first translated in to HTML like this:
/// ```html
///      <UL>  <!--1--!>
///         <LI>
///             list line 1
///         </LI>
///         <UL>
///         </UL> <!--2--!>
///         <LI>
///             list line 2
///         </LI>
///     </UL>
/// ```
///
/// Then the standard merge nodes call is made which recognizes the same format, and changes to:
///
/// ```html
///          <UL> <!--1-->  <!-- doc-node with line FORMAT, but EMPTY !! content -->
///             <LI> <!-- doc-node with line FORMAT -->
///                 list line 1  <!-- doc-node(s) with line content -->
///             </LI>
///             <LI>
///                 list line 2
///             </LI>
///         </UL>
///```
///
/// The `<UL>` node will be a doc-node in the doc-tree. This allows the normal functions to operate.
/// But when creating the delta document from the doc-tree, we find a operation with length = 0;
/// Hence it will not appear in any of the operations list when extracting the delta
///
/// Deleting format deletes the format for the whole "content" contained in the block ...
/// Taking `[*]` as the cursor position, we delete the format by:
///  - isolatating the `<LI>` element involved
///  - putting everything under the `<LI>` tag back to an `<P>` tag
///
/// ```html
///     <UL>
///        <LI>
///            list line 1
///        </LI>
///        <LI>
///           list [*] line 2
///        </LI>
///        <LI>
///            list line 3
///        </LI>
///    </UL>
///    <UL>
///       <LI>
///          list line 1
///       </LI>
///    </UL>
///       <P>
///          list [*] line 2
///       </P>
///    <UL>
///       <LI>
///           list line 3
///       </LI>
///    </UL>
/// ```
///
///
#[allow(non_camel_case_types)]
pub struct ListBlock {
    block_name: &'static str,
    parent_tag: &'static str,
    child_tag: &'static str,
    attr_val: &'static str,
}

impl ListBlock {
    pub fn new_ul() -> ListBlock {
        block_format::initialise();
        ListBlock {
            block_name: NAME_UL_BLOCK,
            parent_tag: UL_TAG,
            child_tag: LI_TAG,
            attr_val: LIST_BULLET,
        }
    }
    pub fn new_ol() -> ListBlock {
        block_format::initialise();
        ListBlock {
            block_name: NAME_OL_BLOCK,
            parent_tag: OL_TAG,
            child_tag: LI_TAG,
            attr_val: LIST_ORDERED,
        }
    }
}

impl ListBlock {
    /// Similar to the link format we create aa common parent with length is 0 to collect the list
    /// list_tags (type impl FormatTrait) is needed to get to the correct parent & child tags
    /// formatter (type impl FormatTrait) is the pointer to the format in the Registry; we point to that one in the created doc_nodes
    fn create_list_node(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<(Arc<DocumentNode>, Arc<DocumentNode>)> {
        //<UL>
        let ul_el = DomElement::new(self.parent_tag);
        block_format::apply(&ul_el, operation.get_attributes())?;
        let mut op = DeltaOperation::insert("");
        op.set_attributes(operation.get_attributes().clone());
        let ul_doc_node = DocumentNode::new_element(ul_el, formatter.clone());
        ul_doc_node.set_operation(op);

        //<LI>
        let li_el = DomElement::new(self.child_tag);
        let li_doc_node = DocumentNode::new_element(li_el, formatter.clone());
        li_doc_node.set_operation(operation);
        let li_doc_node_ptr = Arc::new(li_doc_node);

        //Link UL -> LI
        let ul_doc_node_ptr = Arc::new(ul_doc_node);
        append(&ul_doc_node_ptr, li_doc_node_ptr.clone());

        //Return the pair so that the calling method may use either one without traversing the tree again
        return Ok((ul_doc_node_ptr, li_doc_node_ptr));
    }

    /// Splits of a LI node such that it is the only LI element in an UL parent block.<br>
    /// Returned: UL node --> parent UL block of this single LI
    fn split_li_in_own_ul(&self, li_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        let mut ul_node = li_node.get_parent().unwrap();

        //isolate the <LI> block so that it sits in its own <UL> block
        if let Some(next) = next_sibling(&li_node) {
            split_block_before_child(&ul_node, &next)?;
        }
        if let Some(_prev) = prev_sibling(&li_node) {
            ul_node = split_block_before_child(&ul_node, &li_node)?.clone();
        }
        assert_eq!(ul_node.child_count(), 1); //expecting isolated node !!

        Ok(ul_node.clone())
    }

    /// Merging of UL nodes:
    ///  - merges 2 next sibling nodes if `<UL>`
    ///  - merges 2 previous sibling nodes if `<UL>`
    /// No merge if the UL nodes happen to have different attributes. (such as indentation)
    ///
    /// It is up to the caller to update the cursor if any is needed ...<br>
    ///
    /// Returns: resulting merged UL node.<br>
    fn merge_ul_nodes(&self, ul_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        //now we merge if needed
        let op = ul_node.get_operation();
        if let Some(next) = next_sibling(&ul_node) {
            if next.get_doc_dom_node().get_node_name() == self.parent_tag
                && op == next.get_operation()
            {
                merge_block_node(&ul_node, &next)?;
                if let Some(nxt_nxt) = next_sibling(&ul_node) {
                    if nxt_nxt.get_doc_dom_node().get_node_name() == self.parent_tag
                        && op == nxt_nxt.get_operation()
                    {
                        merge_block_node(&ul_node, &nxt_nxt)?;
                    }
                }
            }
        }

        let mut ret = ul_node.clone();
        if let Some(prev) = prev_sibling(&ul_node) {
            if prev.get_doc_dom_node().get_node_name() == self.parent_tag
                && op == prev.get_operation()
            {
                ret = prev.clone();
                merge_block_node(&prev, &ul_node)?;
                if let Some(prv_prv) = prev_sibling(&prev) {
                    if prv_prv.get_doc_dom_node().get_node_name() == self.parent_tag
                        && op == prv_prv.get_operation()
                    {
                        ret = prv_prv.clone();
                        merge_block_node(&prv_prv, &prev)?;
                    }
                }
            }
        }
        return Ok(ret);
    }
}

impl FormatTait for ListBlock {
    fn create(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let (ul_node, _) = self.create_list_node(operation, formatter)?;
        return Ok(ul_node);
    }

    fn format_name(&self) -> &'static str {
        self.block_name
    }

    fn is_text_format(&self) -> bool {
        false
    }

    fn block_remove_attr(&self) -> Attributes {
        let mut attr = Attributes::default();
        attr.insert(LIST_ATTR_KEY, Null);
        attr
    }

    fn applies(&self, delta: &DeltaOperation) -> Result<bool> {
        if delta.insert_value().is_string() {
            let s = delta.insert_value().str_val()?;
            if s == "\n" || s == "" {
                //allow "" to detect the format given to an UL doc_node
                if let Some(AttrVal::String(l_type)) = delta.get_attributes().get(LIST_ATTR_KEY) {
                    return Ok(self.attr_val.eq(l_type));
                }
            }
        }
        return Ok(false);
    }

    fn apply_line_attributes(
        &self,
        doc_node: &Arc<DocumentNode>,
        attr: &Attributes,
        _formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        apply_attributes(doc_node, attr)
    }

    fn drop_line_attributes(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        drop_attributes(doc_node)
    }

    fn clone_doc_node(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        let name = doc_node.get_doc_dom_node().get_node_name();
        let node = if name == self.child_tag {
            let li_el = DomElement::new(self.child_tag);
            let li_doc_node = Arc::new(DocumentNode::new_element(
                li_el,
                doc_node.get_formatter().clone(),
            ));
            li_doc_node.set_operation(doc_node.get_operation());

            // let p_parent = doc_node.get_parent().unwrap();
            // insert_after(&p_parent, &doc_node, &ul_doc_node);

            li_doc_node
        } else if name == UL_TAG || name == OL_TAG {
            // in case we are splitting text
            let ul_el = DomElement::new(&name);
            let operation = doc_node.get_operation();
            block_format::apply(&ul_el, operation.get_attributes())?;
            let mut op = DeltaOperation::insert("");
            op.set_attributes(operation.get_attributes().clone());
            let ul_doc_node = Arc::new(DocumentNode::new_element(
                ul_el,
                doc_node.get_formatter().clone(),
            ));
            ul_doc_node.set_operation(op);

            // let p_parent = doc_node.get_parent().unwrap();
            // insert_after(&p_parent, &doc_node, &ul_doc_node);

            ul_doc_node
        } else {
            assert_eq!(
                "LIST: Hey why are we in this branch of the IF statement TEXT",
                ""
            );
            //create text format ...
            let operation = doc_node.get_operation();
            let format = doc_node.get_formatter();
            format.create(operation, format.clone())?
        };

        return Ok(node);
    }

    fn split_leaf(&self, _cursor: &Cursor) -> Result<()> {
        panic!("ListFormat::split_leaf() - Error. ");
    }

    fn is_same_format(&self, left: &Arc<DocumentNode>, right: &Arc<DocumentNode>) -> bool {
        if let Some(AttrVal::String(l)) = left.get_operation().get_attributes().get(LIST_ATTR_KEY) {
            if let Some(AttrVal::String(r)) =
                right.get_operation().get_attributes().get(LIST_ATTR_KEY)
            {
                return l.eq(r);
            }
        }
        return false;
    }

    /// The transformation returns a pointer to the new `<UL>` node. But the cursor points to either
    /// an empty `<LI>` block or some `<text>`.
    fn block_transform(
        &self,
        cursor: &Cursor,
        block_node: &Arc<DocumentNode>,
        delta: DeltaOperation,
        format: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let update_cursor = &cursor.get_doc_node() == block_node;
        //assert!( (update_cursor && block_node.child_count()==0) || block_node.child_count()>0 );
        assert!((update_cursor && block_node.child_count() == 0) || !update_cursor);

        let (ul_node, li_node) = self.create_list_node(delta, format.clone())?;

        let parent = block_node.get_parent().unwrap();
        insert_before(&parent, block_node, ul_node.clone());
        unlink(&parent, block_node);

        //Empty block node
        if block_node.child_count() == 0 {
            if update_cursor {
                cursor.set_at(&li_node, 0);
            }
            return Ok(ul_node);
        }

        //Non empty block node
        for c in block_node.get_children().iter() {
            unlink(block_node, c);
            append(&li_node, c.clone());
        }
        return self.merge_ul_nodes(&ul_node);
    }

    /// Split will point to a UL block, so we expect that as input here
    fn un_block_transform(
        &self,
        cursor: &Cursor,
        li_node: &Arc<DocumentNode>,
    ) -> Result<Arc<DocumentNode>> {
        //expecting <LI> block ...
        //error!("LIST un_block_transform(): {}", cursor);
        assert_eq!(li_node.get_doc_dom_node().get_node_name(), self.child_tag);
        let update_cursor = &cursor.get_doc_node() == li_node;
        //If we do not update the cursor, we should not care about the # children of the LI element
        //assert!( (update_cursor && li_node.child_count()==0) || li_node.child_count()>0 );
        assert!((update_cursor && li_node.child_count() == 0) || !update_cursor);

        let mut ul_node = li_node.get_parent().unwrap();
        let parent = ul_node.get_parent().unwrap();

        //isolate the <LI> block so that it sits in its own <UL> block
        if let Some(next) = next_sibling(&li_node) {
            split_block_before_child(&ul_node, &next)?;
        }
        if let Some(_prev) = prev_sibling(&li_node) {
            ul_node = split_block_before_child(&ul_node, &li_node)?.clone();
        }
        assert_eq!(ul_node.child_count(), 1); //expecting isolated node !!

        //now transform the solitary <LI> block in to a <P> block

        //Create new <P> block and add to tree
        let idx = ul_node.my_index_as_child().unwrap();
        let mut attr = ul_node.get_operation().get_attributes().clone();
        attr.remove(LIST_ATTR_KEY);
        let op = DeltaOperation::insert_attr("\n", attr);
        let p_node = P_FORMAT.create(op, P_FORMAT.clone())?;
        insert_at_index(&parent, idx, p_node.clone());

        unlink(&ul_node, &li_node);
        unlink(&parent, &ul_node);

        //empty <LI> block turns into empty <P>
        if update_cursor {
            cursor.set_at(&p_node, 0);
            return Ok(p_node);
        }

        //Non empty block
        let children = li_node.get_children();
        for i in 0..li_node.child_count() {
            let n = children.get(i).unwrap();
            unlink(&li_node, n);
            append(&p_node, n.clone());
        }
        return Ok(p_node);
    }

    fn delete_leaf_segment(
        &self,
        _doc_node: &Arc<DocumentNode>,
        _at: usize,
        _length: usize,
    ) -> Result<()> {
        panic!(
            "ListFormat::delete() - Error. Block has length 1, so use the other delete function..."
        );
    }

    fn delete_node(&self, block_node: &Arc<DocumentNode>) {
        assert_eq!(
            block_node.get_doc_dom_node().get_node_name(),
            self.child_tag
        );
        let ul_node = block_node.get_parent().unwrap();
        delete_node(block_node);
        if ul_node.child_count() == 0 {
            delete_node(&ul_node);
        }
    }

    /// Returns the UL doc_node with one single block `<LI>` node which contains the doc_node from
    /// the input.<br>
    /// returns: `<UL>` node
    fn isolate(&self, li_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        assert_eq!(li_node.get_doc_dom_node().get_node_name(), self.child_tag);
        self.split_li_in_own_ul(&li_node)
    }

    /// When we are merging, but we want to merge using a format from the `<UL>` formats.
    /// So how to get that node which can be used as a pivot ...
    fn try_merge(&self, _cursor: &Cursor, li_node: &Arc<DocumentNode>) -> Result<()> {
        //assert_eq!(li_node.get_doc_dom_node().get_node_name(), self.child_tag );
        let ul_node = if li_node.get_doc_dom_node().get_node_name() == self.child_tag {
            li_node.get_parent().unwrap()
        } else {
            li_node.clone()
        };

        self.merge_ul_nodes(&ul_node)?;
        Ok(())
    }
}
