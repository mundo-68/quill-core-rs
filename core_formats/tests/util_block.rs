use anyhow::Result;
use core_formats::paragraph::Pblock;
use core_formats::text_formatter::TextFormat;
use core_formats::util::block::{block_transform, un_block_transform};
use core_formats::util::node_morph::split_text_and_block_at_cursor;
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use dom::dom_element::DomElement;
use log::error;
use node_tree::cursor::Cursor;
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::append;
use node_tree::format_trait::FormatTait;
use node_tree::tree_traverse::{first_node, prev_leaf};
use op_transform::doc_root::DocumentRoot;
use std::sync::Arc;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// The util_block module does not add `<br/>`. So here we do not add it to the tests

struct HeadingSkeleton {}
impl HeadingSkeleton {
    pub fn new() -> Self {
        HeadingSkeleton {}
    }
}

impl FormatTait for HeadingSkeleton {
    fn create(
        &self,
        operation: DeltaOperation,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        let element = DomElement::new("H1");
        let dn = Arc::new(DocumentNode::new_element(element, formatter.clone()));
        dn.set_operation(operation);
        return Ok(dn);
    }

    fn format_name(&self) -> &'static str {
        todo!()
    }

    fn is_text_format(&self) -> bool {
        false
    }

    fn block_remove_attr(&self) -> Attributes {
        todo!()
    }

    fn applies(&self, _delta: &DeltaOperation) -> Result<bool> {
        todo!()
    }

    fn apply_line_attributes(
        &self,
        _doc_node: &Arc<DocumentNode>,
        _attr: &Attributes,
        _formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        todo!()
    }

    fn drop_line_attributes(&self, _doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        todo!()
    }

    fn split_leaf(&self, _cursor: &Cursor) -> Result<()> {
        todo!()
    }

    fn is_same_format(&self, _left: &Arc<DocumentNode>, _right: &Arc<DocumentNode>) -> bool {
        todo!()
    }

    fn block_transform(
        &self,
        cursor: &Cursor,
        block_node: &Arc<DocumentNode>,
        delta: DeltaOperation,
        format: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<Arc<DocumentNode>> {
        block_transform(block_node, delta, format, cursor)
    }

    fn un_block_transform(
        &self,
        cursor: &Cursor,
        block_node: &Arc<DocumentNode>,
    ) -> Result<Arc<DocumentNode>> {
        un_block_transform(block_node, cursor)
    }

    fn delete_leaf_segment(
        &self,
        _doc_node: &Arc<DocumentNode>,
        _at: usize,
        _length: usize,
    ) -> Result<()> {
        todo!()
    }

    fn delete_node(&self, _doc_node: &Arc<DocumentNode>) {
        todo!()
    }

    fn isolate(&self, _doc_node: &Arc<DocumentNode>) -> Result<Arc<DocumentNode>> {
        todo!()
    }

    fn try_merge(&self, _cursor: &Cursor, _block_node: &Arc<DocumentNode>) -> Result<()> {
        todo!()
    }
}

#[wasm_bindgen_test]
fn block_transform_test() -> Result<()> {
    let p_format = Arc::new(Pblock::new());
    let t_format = Arc::new(TextFormat::new());
    let h_format: Arc<HeadingSkeleton> = Arc::new(HeadingSkeleton::new());

    let mut attr = Attributes::default();
    attr.insert("heading", 1);

    //------------------------------------------
    // create doc
    //------------------------------------------
    let doc = DocumentRoot::new("block_transform_test");
    doc.append_to_body();
    //doc.open()?; --> we are not applying any delta operation !
    let root = doc.get_root();

    let delta = DeltaOperation::insert("\n");
    let par = p_format.create(delta, p_format.clone())?;
    append(&root, par.clone());

    let delta = DeltaOperation::insert("Heading text");
    let t = t_format.create(delta, t_format.clone())?;
    append(&par, t.clone());

    let delta = DeltaOperation::insert("\n");
    let par2 = p_format.create(delta, p_format.clone())?;
    append(&root, par2.clone());

    let expect = r#"<p>Heading text</p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------
    // block transform
    //------------------------------------------
    let cursor = Cursor::new();
    cursor.set_at(&par2, 0);
    let delta = DeltaOperation::insert_attr("\n", attr);
    h_format.block_transform(&cursor, &par, delta, h_format.clone())?;

    let expect = r#"<h1>Heading text</h1><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------
    // UN-block transform
    //------------------------------------------
    let children = root.get_children();
    let header = children.get(0).unwrap();
    error!("Header length = {}", header.op_len());

    //split text so that we can see the ordering of blocks
    let children = header.get_children();
    cursor.set_at(&children.get(0).unwrap(), 7);
    //error!("showing cursor before split {}", &cursor);
    split_text_and_block_at_cursor(&cursor)?;
    let expect = r#"<h1>Heading</h1><h1> text</h1><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    //error!("showing cursor after split  {}", &cursor);

    //will fail if we got the cursor post condition false
    let first = prev_leaf(&cursor.get_doc_node()).unwrap();
    cursor.set_after(&first);
    let block = cursor.get_doc_node().get_parent().unwrap();

    un_block_transform(&block, &cursor)?;
    let expect = r#"<p>Heading</p><h1> text</h1><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    Ok(())
}

#[wasm_bindgen_test]
fn op_inset_edge_test() -> Result<()> {
    let p_format = Arc::new(Pblock::new());
    let t_format = Arc::new(TextFormat::new());

    let mut attr = Attributes::default();
    attr.insert("heading", 1);

    //------------------------------------------
    // create doc
    //------------------------------------------
    let doc = DocumentRoot::new("op_inset_edge_test");
    doc.append_to_body();
    //doc.open()?; --> we are not applying any delta operation
    let root = doc.get_root();

    let delta = DeltaOperation::insert("\n");
    let par = p_format.create(delta, p_format.clone())?;
    append(&root, par.clone());

    let delta = DeltaOperation::insert("Heading text");
    let t = t_format.create(delta, t_format.clone())?;
    append(&par, t.clone());

    let delta = DeltaOperation::insert("\n");
    let par2 = p_format.create(delta, p_format.clone())?;
    append(&root, par2.clone());

    let expect = r#"<p>Heading text</p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);

    //------------------------------------------
    // block transform
    //------------------------------------------
    let root = doc.get_root();
    let left_text = first_node(&root);
    let cursor = Cursor::new();
    cursor.set_before(&left_text);
    split_text_and_block_at_cursor(&cursor)?;
    //error!("insert_new_block - {}", &cursor);

    let expect = r#"<p></p><p>Heading text</p><p></p>"#;
    assert_eq!(doc.as_html_string(), expect);
    //error!("{}", &cursor);
    Ok(())
}
