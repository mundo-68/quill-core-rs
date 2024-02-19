// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::cursor::Cursor;
use crate::doc_node::DocumentNode;
use anyhow::Result;
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use std::sync::Arc;

/// # FormatTait
///
/// All formats supported shall implement this trait.
/// The operational transform functions will use this trait to create the HTML document from delta-operations
/// The trait implementation shall not change the `delta_operation` in the `document_node`
pub trait FormatTait {
    /// # create()
    ///
    /// --------------------------------------------------------------
    /// MUST implement by the implementing `FormatTait`
    ///--------------------------------------------------------------
    /// Creates a `DocumentNode`, and a representation of that node in the HTML DOM.
    fn create(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>>;

    /// # format_name()
    ///
    /// Returns the name of the format. This is the same name which is used to register in the
    /// `Registry`. This name shall be unique for all implementers of the `FormatTait`.
    fn format_name(&self) -> &'static str;

    /// # is_text_format()
    ///
    /// A leaf node is a node without children. Here, this means it is a "Text format" instead
    /// of a "block_format".
    ///
    /// This is a bit of the similar as `is_block_format()`. But without having the `DeltaOperation`
    /// as input.
    fn is_text_format(&self) -> bool;

    /// # block_remove_attr()
    ///
    /// Returns an attribute structure which switches off all the attributes that define
    /// the block format.
    ///
    /// This means that the attribute `keys` are defined by the formatter, but the values
    /// are defined to be `AttrVal::Null`
    fn block_remove_attr(&self) -> Attributes;

    /// # applies()
    ///
    /// Returns true if this `FormatTait` handles the given d`DeltaOperation`.
    ///
    /// This function allows the operational transform functions handling the `DeltaOpoeration` to
    /// stay agnostic of all known `FormatTait` implementations which may or may not be in the
    /// `Registry`.
    fn applies(&self, delta: &DeltaOperation) -> Result<bool>;

    /// # apply_line_attributes()
    ///
    /// Applies format to the whole `DocumentNode` pointed to by the cursor
    ///
    /// The cursor will be updated when the document node pointed to is destroyed
    fn apply_line_attributes(
        &self,
        doc_node: &Arc<DocumentNode>,
        attr: &Attributes,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>>;

    /// # drop_line_attributes()
    ///
    /// Removes all formatting from the whole `DocumentNode` pointed to by the cursor.
    /// This will change the document node in to a standard `TEXT` node.
    ///
    /// The cursor will be updated when the document node pointed to is destroyed
    fn drop_line_attributes(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>>;

    ///--------------------------------------------------------------
    /// MAY implement by the implementing `FormatTait`
    ///--------------------------------------------------------------

    /// # clone_doc_node()
    ///
    /// returns a clone of the document node.<br>
    /// Implementation note: child nodes are NOT cloned.
    /// FIXME: Can we do without this member?
    fn clone_doc_node(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        let formatter = doc_node.get_formatter();
        let clone = formatter.create(doc_node.get_operation(), formatter.clone())?;
        Ok(clone)
    }

    /// # split_leaf()
    ///
    /// Split ONLY the leaf node to which the cursor points.
    ///
    /// Example with plain text, splits in to 2 `<text>` nodes:
    ///
    ///  - split @3: `<P>Hello world</P>` results in `<P>[Hel][{*}lo world]</P>`
    ///
    /// Example with more complex text element, like `<a>` splits into 2x `<a>` element:
    ///
    ///  - split @3: `<a href="">hello world</a>` results in `<a href="">hel</a> <a href="">{*}lo world</a>`
    ///
    /// where `{*}` denotes the cursor position.
    ///
    /// Mandatory post condition: the cursor position is `BEFORE` the right hand leaf node
    /// This post condition allows the cursor to move correctly after `insert(\n)` operations
    ///
    /// Default implementation assumes splitting normal text node from first example
    /// Post condition: Cursor position is BEFORE the right hand side created node
    fn split_leaf(&self, cursor: &Cursor) -> Result<()>;

    /// # is_same_format()
    ///
    /// Returns true when both formats are identical, in such a way that the
    /// 2 nodes should be merged.
    ///
    /// It is not enough to only check the format; in most cases the attributes need to match too.
    /// To retrieve the attributes, the `DocumentNode` is needed. Hence we need both left, and
    /// right hand `DocumentNode`.
    fn is_same_format(&self, left: &Arc<DocumentNode>, right: &Arc<DocumentNode>) -> bool;

    /// # block_transform()
    ///
    /// Transforms the block node in to a proper block formatted with the desired format.
    /// The format may contain more formatting to be (un)applied ...
    ///
    /// Input:
    ///  - block_node : children of this block will be children of the new block
    ///  - format_prt: We need a Arc-pointer to the format ... which is not SELF that is the a normal memory pointer
    fn block_transform(
        &self,
        cursor: &Cursor,
        block_node: &Arc<DocumentNode>,
        delta: DeltaOperation,
        format_ptr: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>>;

    /// # un_block_transform()
    ///
    /// Removes the block transform formatting from the current block, and transforms it into a
    /// standard `<P>` paragraph block.
    ///
    /// For list formatting, or tables, this can be a bit complicated. Hence this is part of the
    /// `FormatTrait`.
    fn un_block_transform(
        &self,
        cursor: &Cursor,
        block_node: &Arc<DocumentNode>,
    ) -> Result<Arc<DocumentNode>>;

    /// # delete_leaf_segment()
    ///
    /// Deletes text in a block, but NOT the whole `DocumentNode`
    fn delete_leaf_segment(
        &self,
        doc_node: &Arc<DocumentNode>,
        at: usize,
        length: usize,
    ) -> Result<()>;

    /// # delete_node()
    ///
    /// Deletes one whole node:
    ///  - in case of a block node, then block node shall be devoid of children before calling this
    fn delete_node(&self, doc_node: &Arc<DocumentNode>);

    /// # isolate()
    ///
    /// Isolates a document node from its environment.
    /// Some formats are nested, requiring zero length `DocumentNode` to contain the structure.
    /// When transforming these document nodes it is mostly easier to isolate a `DocumentNode`
    /// and then to merge them later.
    ///
    /// Example for a text node:
    /// ```html
    ///   <A>hello<S>{*}sweet</S><EM> world</EM></A>
    /// ```
    /// will become after isolate()
    /// ```bash
    ///   <A>hello</A><{*}A><S>sweet</S></A><A><EM> world</EM></A>
    /// ```
    /// Also note that the cursor returned follows the upper most doc node...
    ///
    /// Example for a block node:
    /// ```html
    ///   <UL>
    ///      <LI>line 1</LI>
    ///      {*}<LI>line 2</LI>
    ///      <LI>line 3</LI>
    ///   </UL>
    /// ```
    /// will become after isolate()
    /// ```html
    ///   <UL>
    ///      <LI>line 1</LI>
    ///   </UL><{*}UL>
    ///      <LI>line 2</LI>
    ///   </UL><UL>
    ///      <LI>line 3</LI>
    ///   </UL>
    /// ```
    fn isolate(&self, doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>>;

    /// # try_merge()
    ///
    /// Sometimes an edit leaves the document in a state where the DOM format is not standard.
    /// For example leaving 2 lists, then the merge should merge both `<LI>` elements in one `<UL>`:
    /// ```html
    ///      <UL><LI>hello</LI></UL><UL><LI>world</LI></UL>
    /// returns
    ///      <UL><LI>hello</LI><LI>world</LI></UL>
    /// ```
    /// The cursor may be pointing to any one of the text labels `hello` or `world`. Both should merge.
    /// This function should check left and right, so it may become a 3 way merge ...
    ///
    /// The design expects that if there is a merge possible, then the same `FormatTait`
    /// should apply.
    ///
    /// If no merge is possible, then that is not an error; nothing shall be changed to the document
    /// state.
    ///
    /// In general, text node shall use the cursor to merge. Block node shall use the
    /// input block node to merge. Both block and text formats will update the cursor when required...
    fn try_merge(&self, cursor: &Cursor, block_node: &Arc<DocumentNode>) -> Result<()>;
}

/// # RootFormat
///
/// Dumbo structure for root node<br>
/// But it may also be used for testing purposes
pub struct RootFormat {
    pub name: &'static str,
}
impl RootFormat {
    pub fn new(name: &'static str) -> Self {
        RootFormat { name }
    }
}
static RT_FORMAT_ERROR: &str = "Seems a programming error; calling a RootFormat method.";
impl FormatTait for RootFormat {
    fn create(
        &self,
        _operation: DeltaOperation,
        _formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        panic!("{:?} -- You called: {:?}", RT_FORMAT_ERROR, "create");
    }
    fn format_name(&self) -> &'static str {
        self.name
    }
    //we return false here, so that tests on the root_node will show we have a block format
    //at least needed for node_tree::doc_node::is_empty_block()
    fn is_text_format(&self) -> bool {
        false
    }

    fn block_remove_attr(&self) -> Attributes {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "block_remove_attr");
    }
    fn applies(&self, _delta: &DeltaOperation) -> Result<bool> {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "applies");
    }
    fn apply_line_attributes(
        &self,
        _doc_node: &Arc<DocumentNode>,
        _attr: &Attributes,
        _formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "apply_attributes");
    }
    fn drop_line_attributes(&self, _doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "drop_attributes");
    }
    fn clone_doc_node(&self, _doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "clone_doc_node");
    }
    fn split_leaf(&self, _cursor: &Cursor) -> Result<()> {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "split_leaf");
    }
    fn is_same_format(&self, _left: &Arc<DocumentNode>, _right: &Arc<DocumentNode>) -> bool {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "is_same_format");
    }
    fn block_transform(
        &self,
        _cursor: &Cursor,
        _block_node: &Arc<DocumentNode>,
        _delta: DeltaOperation,
        _format_ptr: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "block_transform");
    }
    fn un_block_transform(
        &self,
        _cursor: &Cursor,
        _block_node: &Arc<DocumentNode>,
    ) -> Result<Arc<DocumentNode>> {
        panic!(
            "{} -- You called: {}",
            RT_FORMAT_ERROR, "un_block_transform"
        );
    }
    fn delete_leaf_segment(
        &self,
        _doc_node: &Arc<DocumentNode>,
        _at: usize,
        _length: usize,
    ) -> Result<()> {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "delete");
    }
    fn delete_node(&self, _doc_node: &Arc<DocumentNode>) {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "delete_node");
    }
    fn isolate(&self, _doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "isolate");
    }
    fn try_merge(&self, _cursor: &Cursor, _block_node: &Arc<DocumentNode>) -> Result<()> {
        panic!("{} -- You called: {}", RT_FORMAT_ERROR, "try_merge");
    }
}
