use crate::auto_soft_break::AutomaticSoftBreak;
use crate::error::Error::DocumentNotOpenForEdit;
use crate::registry::Registry;
use crate::{init_log, op_delete, op_insert, op_retain, set_panic_hook};
use anyhow::Result;
use delta::delta::Delta;
use delta::operations::DeltaOperation;
use delta::types::ops_kind::OpKind;
use dom::dom_element::{get_dom_element_by_id, DomElement};
use log::{trace, Level};
use node_tree::cursor::Cursor;
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::{append, unlink};
use node_tree::format_trait::RootFormat;
use node_tree::tree_traverse::{first_node, last_block_node, next_node};
use node_tree::EDITOR_CLASS;
use std::sync::Arc;
use web_sys::Node;

static CONTAINTER_CLASS: &str = "ql-container";
static STYLE_SNOW: &str = "ql-snow";
static DIV_ELEMENT: &str = "DIV";

/// The editor mode allows the client to change behaviour based on the
/// state of the editor.
#[derive(Clone, PartialEq)]
pub enum EditorMode {
    Edit,   //responding to user input
    Read,   //only reading
    Closed, //Nothing to show ..
}

const DOC_ROOT_FORMAT: &str = "DOC_ROOT_FORMAT";

///  # DocumentRoot
///
/// The doc-root collects:
///  - a document editor state
///  - a registry with the supported DeltaOperation types
///  - a cursor pointing to some location in the document
///  - a root element which points to the HTML DOM node which contains all of the content of this document
///
/// The functions provided on this struct allow for basic control of the document:
///  - open / close
///  - link to a root HTML DOM node
///
/// Implementation note: The root document node has no parent. All other nodes shall have a parent.
/// ```html
/// <div class="ql-container ql-snow" id="some_id" >
///    <div class="ql-editor" contenteditable="true" >
///         <p><br></p>
///    </div>
/// </div>
/// ```
/// FIXME: Remove clone derive from document root. --> we should not need that
#[derive(Clone)]
pub struct DocumentRoot {
    mode: EditorMode,
    cursor: Cursor,               //current location of the cursor
    container: Arc<DocumentNode>, //container for root element
    root: Arc<DocumentNode>,      //container for browser content
}

impl DocumentRoot {
    /// Just starting a new document, not attached to the HTML DOM.
    /// Creating a new DocRoot only<br>
    /// We need an ID:  `<DIV id="xx"></div>` is the marker for the root element
    ///
    /// The root element shall NEVER get a 2nd child --> get_root() depends on it.
    pub fn new(id: &str) -> Self {
        if id == "" {
            panic!("root document node must have a unique Id, found empty ID")
            //Fixme
            //return Err(DocumentRootUniqueId.into())
        }

        let container_element = DomElement::new(DIV_ELEMENT);
        container_element.set_attribute("id", id);
        //dom_element.set_class( EDITOR_CLASS );  // --> should go in own container + content editable
        container_element.set_class(CONTAINTER_CLASS);
        container_element.set_class(STYLE_SNOW);

        let root_element = DomElement::new(DIV_ELEMENT);
        root_element.set_class(EDITOR_CLASS);
        container_element.append_child(root_element.node());

        let container = DocumentNode::new_element(
            container_element,
            Arc::new(RootFormat::new(DOC_ROOT_FORMAT)),
        );

        let root =
            DocumentNode::new_element(root_element, Arc::new(RootFormat::new(DOC_ROOT_FORMAT)));

        // set some WASM browser utilities
        set_panic_hook();
        #[cfg(test)]
        init_log(Level::Debug);

        DocumentRoot {
            mode: EditorMode::Read,
            //registry: Arc::new(RefCell::new(Registry::default())),
            cursor: Cursor::new(),
            container: Arc::new(container),
            root: Arc::new(root),
        }
    }

    pub fn set_log_level(level: Level) {
        init_log(level);
    }

    ///Container for all edit content
    pub fn get_root(&self) -> &Arc<DocumentNode> {
        &self.root
    }

    // /// Returns the block format operation for this document node
    // /// If the input doc_node is already a block format, then we return the delta operation for
    // /// the input doc_node, not!! its parent block
    // pub fn block_operation_from_doc_node(doc_node: &Arc<DocumentNode>) -> DeltaOperation {
    //     let block = if !doc_node.is_leaf() {
    //         doc_node.clone()
    //     } else {
    //         let mut parent = doc_node.clone();
    //         loop {
    //             parent = parent.get_parent().unwrap();
    //             if !parent.is_leaf() {
    //                 break;
    //             } //break the loop
    //         }
    //         parent
    //     };
    //
    //     return block.get_operation();
    // }
}

/// Showing the document in the HTML DOM
impl DocumentRoot {
    /// Binds directly to the body of the HTML document. Note: We will APPEND !!
    pub fn append_to_body(&self) {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("document should have a body");

        body.append_with_node_1(self.container.get_html_node())
            .expect("Document:append_to_body()");
    }

    /// Cleans up the document node in the HTML dom tree
    pub fn detach(&mut self) {
        match self.get_root().get_dom_element().unwrap().get_parent() {
            Some(p) => {
                p.remove_child(self.container.get_html_node())
                    .expect("Document:detach()");
            }
            _ => {}
        }
    }

    /// Binds to a given HTML dom node
    pub fn bind_to(&self, root_parent: &Node) {
        root_parent
            .append_child(self.container.get_html_node())
            .expect("Document:bind_to()");
    }

    /// Binds to a given HTML dom node
    pub fn bind_to_id(&self, id: &str) {
        let el = get_dom_element_by_id(id).unwrap();
        el.append_child(self.container.get_html_node());
    }
}

/// Document root interface which allows manipulation of the document itself.
impl DocumentRoot {
    pub fn get_mode(&self) -> &EditorMode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: EditorMode) {
        self.edit_mode(mode);
    }

    /// Changes the document state. This fits nicely with the DOM state `content editable`.
    fn edit_mode(&mut self, mode: EditorMode) {
        let el = self.root.get_dom_element().unwrap();
        match &mode {
            EditorMode::Edit => {
                el.set_attribute("contenteditable", "true");
            }
            EditorMode::Read => {
                el.remove_attribute("contenteditable");
            }
            EditorMode::Closed => {
                el.remove_attribute("contenteditable");
            }
        }
        self.mode = mode;
    }

    /// Open a DeltaDocument, and put its content in the selected DOM node
    pub fn open(&mut self) -> Result<()> {
        let registry = Registry::get_ref()?;
        if self.get_mode() != &EditorMode::Closed {
            self.close()
        }
        self.edit_mode(EditorMode::Edit);
        let op = DeltaOperation::insert("\n");
        let format = registry.block_format(&op)?;
        let block = format.create(op, format.clone())?;

        //insert_soft_break() to allow a cursor to appear in an empty <p></p> block
        AutomaticSoftBreak::insert(&block)?;

        //Note: cursor AT(dn,0) for the <P>-block is the default start for an empty document
        //In general if a new empty block is started, then also the cursor should be AT(dn,0)
        //Insert before setting cursor, since the cursor.set_at() tries to set the retain index,
        //which requires the block to be inserted in a valid document
        append(&self.get_root(), block.clone());
        self.get_cursor().set_at(&block, 0);
        Ok(())
    }

    /// Closes the document, and removes all DOM nodes from the HTML context.
    pub fn close(&mut self) {
        for c in self.root.get_children() {
            unlink(&self.root, &c);
        }
        self.edit_mode(EditorMode::Closed);
    }

    /// Collects the DocumentNode tree and renders a valid DeltaDocument
    pub fn to_delta(&self) -> Delta {
        let mut delta = Delta::default();
        let mut dn_o = Some(first_node(&self.root));
        loop {
            if let Some(doc_node) = dn_o {
                delta.push(doc_node.get_operation());
                dn_o = next_node(&doc_node);
            } else {
                break;
            }
        }
        delta
    }
}

/// CURSOR related interface
impl DocumentRoot {
    pub fn get_cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn set_cursor(&self, cursor: &Cursor) {
        self.cursor.from(cursor);
    }

    /// finds the first node in the document, and points to the first character.
    ///
    /// The end point of the cursor is not changed
    pub fn cursor_to_start(&self) {
        let first = first_node(&self.root);
        if first.get_formatter().is_text_format() {
            self.cursor.set_before(&first);
        } else {
            //empty document with probably only a P node
            assert_eq!(first.child_count(), 0);
            self.cursor.set_at(&first, 0);
        }
    }

    /// # cursor_to_end()
    ///
    /// finds the last element in the document and moves the cursor behind that last element.
    ///
    /// Implementation node:<br>
    /// The minimum document contains `<P></P>` as content. Hence the last position
    /// is just before that last `<P>`
    pub fn cursor_to_end(&self) {
        let last = last_block_node(&self.root).unwrap();
        self.cursor.set_cursor_to_doc_node_edge(&last, true);
    }

    /// Sets the cursor to the first node in the document.
    /// FIXME: Should this be a function that operates on the cursor? --> move function to cursor_test?
    pub fn reset_cursor(&self) {
        let node = first_node(&self.root);
        if node.is_leaf() {
            self.cursor.set_before(&node);
        } else {
            self.cursor.set_at(&node, 0);
        }
    }
}

impl DocumentRoot {
    /// # apply_operation()
    ///
    /// This document model ties the operational transform interface and the
    /// HTML representation together.
    ///
    /// We may have put too much in the root module ... but main goal is to
    /// keep the `node_tree` crate un-aware of the `op_transform` to break
    /// a few cyclic dependencies

    /// Applies a single DeltaOperation to the current location of the document cursor
    pub fn apply_operation(&mut self, operation: DeltaOperation) -> Result<()> {
        trace!("Document::apply_operatation({:?})", operation);
        let registry = Registry::get_ref()?;
        if self.mode != EditorMode::Edit {
            return Err(DocumentNotOpenForEdit.into());
        }
        match &operation.get_op_kind() {
            OpKind::Insert(_val) => {
                for o in DocumentRoot::split_text_lines(operation)?.into_iter() {
                    op_insert::insert(self.get_cursor(), o, &registry)?;
                }
            }
            OpKind::Delete(len) => {
                op_delete::delete(self.get_cursor(), *len)?;
            }
            OpKind::Retain(_len) => {
                op_retain::retain(self.get_cursor(), &operation, &registry)?;
            }
        }
        Ok(())
    }

    /// # split_text_lines()
    ///
    /// Splits a DeltaOperation with a potentially multi line text in an vector of DeltaOperation
    /// with single line text. These DeltaOperation are valid Delta, but together they are not
    /// minimal like the delta format prescribes. The only use is that these Delta operations are
    /// more easily translated into HTML.
    ///
    /// Implementation note: string.split() must use  "\n" ...r##"\n"## does not work
    fn split_text_lines(op: DeltaOperation) -> Result<Vec<DeltaOperation>> {
        let mut delta_operations: Vec<DeltaOperation> = Vec::new();

        //if we have an single character
        if op.op_len() == 1 || !op.insert_value().is_string() {
            delta_operations.push(op);
            return Ok(delta_operations);
        }

        //so we are a string operation
        let txt = op.insert_value().str_val()?;
        let paragraphs: Vec<&str> = txt.split('\n').collect();

        for &p in paragraphs.iter() {
            if !p.is_empty() {
                let mut opr = DeltaOperation::insert(p);
                opr.set_attributes(op.get_attributes().clone());
                delta_operations.push(opr);
                let mut opr = DeltaOperation::insert("\n");
                opr.set_attributes(op.get_attributes().clone());
                delta_operations.push(opr);
            } else {
                let mut opr = DeltaOperation::insert("\n");
                opr.set_attributes(op.get_attributes().clone());
                delta_operations.push(opr);
            }
        }

        if delta_operations.len() > 1 {
            delta_operations.pop();
        }
        Ok(delta_operations)
    }

    /// # apply_delta()
    ///
    /// Sets the cursor to the first character in the document,
    /// and inserts a delta document at that location.
    ///
    /// Changes cursor position to the last entry that gets updated.
    ///
    /// This assumes that the delta to be applied is a
    /// delta relative to the start of the document.
    /// If not, use: `apply_delta_from_cursor(...)`
    /// But then you have to make sure the cursor is set right!
    pub fn apply_delta(&mut self, delta: Delta) -> Result<()> {
        if self.mode != EditorMode::Edit {
            return Err(DocumentNotOpenForEdit.into());
        }
        self.reset_cursor();
        for op in delta.get_ops() {
            self.apply_operation(op)?;
        }
        //self.apply_delta_from_cursor(delta)?;
        // The retain index may have changed !!
        // FIXME: Can we update the retain index instead?
        // FIXME: retain_index == sum of insert length + retain length
        // FIXME: This saves us a very expensive operation per delta update ...
        self.cursor.calculate_retain_index();
        Ok(())
    }

    // /// Applies the delta operations in the delta, starting from the current cursor
    // /// position in the current document.
    // ///
    // /// Applying a delta may cause the current HTML DOM selection te become invalid.
    // /// In that case we collapse the selection in the document cursor, and update the
    // /// HTML DOM Range().
    // pub fn apply_delta_from_cursor(&mut self, delta: Delta) -> Result<()> {
    //     if self.mode != EditorMode::Edit {
    //         return Err(DocumentNotOpenForEdit.into());
    //     }
    //     for op in delta.get_ops() {
    //         self.apply_operation(op)?;
    //     }
    //     Ok(())
    // }
}

//#[cfg(all(test, feature = "test_export"))]
impl DocumentRoot {
    pub fn as_html_string(&self) -> String {
        self.root.get_dom_element().unwrap().element().inner_html()
    }

    pub fn as_outer_html_string(&self) -> String {
        self.root.get_dom_element().unwrap().element().outer_html()
    }

    ///Node to use when adding the editor to a HTML text
    pub fn get_container_element(&self) -> &DomElement {
        self.container.get_dom_element().unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::doc_root::DocumentRoot;
    use delta::delta::Delta;

    #[test]
    fn split_text_lines_test() -> anyhow::Result<()> {
        let mut delta = Delta::default();
        delta.insert("\nHello sweet \nworld");
        let v = DocumentRoot::split_text_lines(delta.first().unwrap().clone())?;

        assert_eq!(v.first().unwrap().insert_value().str_val()?, "\n");
        assert_eq!(v.get(1).unwrap().insert_value().str_val()?, "Hello sweet ");
        assert_eq!(v.get(2).unwrap().insert_value().str_val()?, "\n");
        assert_eq!(v.get(3).unwrap().insert_value().str_val()?, "world");
        Ok(())
    }

    #[test]
    fn split_multi_block_formats_test() -> anyhow::Result<()> {
        let mut delta = Delta::default();
        delta.insert("\n\n\n");
        delta.insert("\n");

        let v = DocumentRoot::split_text_lines(delta.first().unwrap().clone())?;

        assert_eq!(v.first().unwrap().insert_value().str_val()?, "\n");
        assert_eq!(v.get(1).unwrap().insert_value().str_val()?, "\n");
        assert_eq!(v.get(2).unwrap().insert_value().str_val()?, "\n");
        assert_eq!(v.get(3).unwrap().insert_value().str_val()?, "\n");
        Ok(())
    }
}
