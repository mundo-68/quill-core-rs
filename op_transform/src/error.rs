// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use delta::operations::DeltaOperation;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Initialise the registry first.")]
    RegistryNotInitialised,
    #[error("Could not lock the registry mutex.")]
    RegistryLockFailed,
    #[error("Can not find format = {fmt} in registry")]
    RegistryNoSuchFormat { fmt: String },
    #[error("Can not find a {tpe} registry entry which matches the operation: = {op:?} ")]
    RegistryNoFormatForOp { tpe: String, op: DeltaOperation },
    #[error("Unexpected cursor position. Programming error? Found position enum = {pos} ")]
    UnexpectedCursorPosition { pos: String },
    #[error("Programming error: The document root must have an unique ID")]
    DocumentRootUniqueId { pos: String },
    #[error("It seems you are changing a document which is not editable")]
    DocumentNotOpenForEdit,
    #[error(
        "You can not delete the last block format in a document. Too many delete operations??"
    )]
    DeletingLastBlock,
    #[error("We seem to be trying delete operations on an empty document!")]
    DeleteOperationOnEmptyDocument,
    #[error("I am at a loss ... it seems that I can not find the next block formatted operation!")]
    CanNotFindNextBlock,
    #[error("I am at a loss ... why did we try to insert an automatic soft-break twice!")]
    DoubleInsertionOfASoftBreak,
    #[error("I am at a loss ... trying to remove an automatic soft-break, where there is none!")]
    CanNotRemoveASoftBreak,
}
