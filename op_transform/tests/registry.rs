use anyhow::Result;
use core_formats::format_const::{NAME_P_BLOCK, NAME_TEXT};
use core_formats::{P_FORMAT, TEXT_FORMAT};
use delta::operations::DeltaOperation;
use op_transform::registry::Registry;
use std::ops::Deref;

#[test]
fn registry_block_format_test() -> Result<()> {
    Registry::init_registry();
    let mut registry = Registry::get_mut_ref()?;
    registry.register_block_fmt(NAME_P_BLOCK, P_FORMAT.deref().clone())?;
    registry.register_line_fmt(NAME_TEXT, TEXT_FORMAT.deref().clone())?;

    let op = DeltaOperation::insert("\n");
    assert!(registry.is_block_fmt(&op)?);

    let op = DeltaOperation::insert(r#"hello \nworld"#);
    assert!(!registry.is_block_fmt(&op)?);
    Ok(())
}
