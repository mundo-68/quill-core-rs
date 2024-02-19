use anyhow::Result;
use core_formats::{P_FORMAT, TEXT_FORMAT};
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use dom::dom_element::DomElement;
use node_tree::dom_doc_tree_morph::append;
use op_transform::doc_root::DocumentRoot;
use op_transform::registry::init_test_registry;

#[cfg(test)]
#[allow(dead_code)] // I get a warning, but in fact the code is used !!
pub fn create_text(doc: &DocumentRoot) -> Result<()> {
    init_test_registry();

    let mut attr = Attributes::default();
    attr.insert("bold", true);

    let root = doc.get_root();

    let delta = DeltaOperation::insert("\n");
    let par = P_FORMAT.create(delta, P_FORMAT.clone())?;
    append(&root, par.clone());

    let delta = DeltaOperation::insert("TEXT_1_1");
    let t = TEXT_FORMAT.create(delta, TEXT_FORMAT.clone())?;
    append(&par, t.clone());

    let delta = DeltaOperation::insert_attr("TEXT_1_2", attr.clone());
    let t = TEXT_FORMAT.create(delta, TEXT_FORMAT.clone())?;
    append(&par, t.clone());

    let delta = DeltaOperation::insert("TEXT_1_3");
    let t = TEXT_FORMAT.create(delta, TEXT_FORMAT.clone())?;
    append(&par, t.clone());

    let delta = DeltaOperation::insert("\n");
    let par = P_FORMAT.create(delta, P_FORMAT.clone())?;
    append(&root, par.clone());

    let delta = DeltaOperation::insert("TEXT_2_1");
    let t = TEXT_FORMAT.create(delta, TEXT_FORMAT.clone())?;
    append(&par, t.clone());

    let delta = DeltaOperation::insert_attr("TEXT_2_2", attr.clone());
    let t = TEXT_FORMAT.create(delta, TEXT_FORMAT.clone())?;
    append(&par, t.clone());

    let delta = DeltaOperation::insert("\n");
    let par = P_FORMAT.create(delta, P_FORMAT.clone())?;
    let sb = DomElement::new("br");
    par.get_dom_element().unwrap().insert_child(0, sb.node());
    append(&root, par.clone());
    Ok(())
}
