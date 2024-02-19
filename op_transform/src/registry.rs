// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::error::Error::{RegistryNoFormatForOp, RegistryNoSuchFormat, RegistryNotInitialised};
use anyhow::Result;
#[cfg(any(test, feature = "test_export"))]
use core_formats::format_const::{NAME_P_BLOCK, NAME_TEXT};
#[cfg(any(test, feature = "test_export"))]
use core_formats::{P_FORMAT, TEXT_FORMAT};
use delta::attributes::Attributes;
use delta::operations::DeltaOperation;
use node_tree::format_trait::FormatTait;
use once_cell::sync::Lazy;
use std::collections::HashMap;
#[cfg(any(test, feature = "test_export"))]
use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
#[cfg(any(test, feature = "test_export"))]
use std::sync::{Mutex, OnceLock};

static REGISTRY: Lazy<RwLock<Registry>> = Lazy::new(|| RwLock::new(Default::default()));

/// # Registry
///
/// Format traits come in 2 varieties, both implementing the FormatTrait:
///
///  - Block formats which are controlling layout vertically
///  - Text formats which are being layout horizontally
///
/// Each format is responsible of detecting if a DeltaOperation is applicable for
/// that format. Some formats may detect when other formats are more applicable.
/// For that reason, formats are checked in a specific order: the order of which
/// they are registered.
///
/// Example of such a dependency: Any text format which is not detected as some
/// other format is by definition a paragraph format.
///
#[derive(Default)]
pub struct Registry {
    block_formats: HashMap<&'static str, Arc<dyn FormatTait + Send + Sync>>, //map resulting in FormatTait
    text_formats: HashMap<&'static str, Arc<dyn FormatTait + Send + Sync>>, //map resulting in FormatTait
    block_order: Vec<&'static str>, //order in which to check the FormatTait
    text_order: Vec<&'static str>,  //order in which to check the FormatTait
}

impl Registry {
    /// We need to call init before doing anything else  ...
    pub fn init_registry() {
        //init_registry();
    }
    pub fn get_ref() -> Result<RwLockReadGuard<'static, Registry>> {
        let Ok(r) = REGISTRY.read() else {
            return Err(RegistryNotInitialised.into());
        };
        Ok(r)
    }

    pub fn get_mut_ref() -> Result<RwLockWriteGuard<'static, Registry>> {
        let Ok(r) = REGISTRY.write() else {
            return Err(RegistryNotInitialised.into());
        };
        Ok(r)
    }

    /// Use with care; you are clearing ALL content of a global data structure
    pub fn clear(&self) -> Result<()> {
        let mut r = Registry::get_mut_ref()?;
        r.block_formats.clear();
        r.text_formats.clear();
        r.block_order.clear();
        r.block_order.clear();
        Ok(())
    }

    /// Vertical text control
    /// The text, and P format must be last in the list --> if not, then we will find the wrong
    /// format since P_BLOCK will trigger on anything. Hence we added an assert!()
    pub fn register_block_fmt(
        &mut self,
        label: &'static str,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<()> {
        self.block_order.push(label);
        self.block_formats.insert(label, formatter);
        Ok(())
    }

    /// Horizontal text control
    /// The text, and P format must be last in the list --> if not, then we will find the wrong
    /// format since P_BLOCK will trigger on anything. Hence we added an assert!()
    pub fn register_line_fmt(
        &mut self,
        label: &'static str,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Result<()> {
        self.text_order.push(label);
        self.text_formats.insert(label, formatter);
        Ok(())
    }

    pub fn block_format_from_name(
        &mut self,
        name: &str,
    ) -> Result<Arc<dyn FormatTait + Send + Sync>> {
        if let Some(format) = self.block_formats.get(name) {
            return Ok(format.clone());
        }
        if self.text_order.is_empty() {
            return Err(RegistryNotInitialised.into());
        }
        return Err(RegistryNoSuchFormat {
            fmt: name.to_string(),
        }
        .into());
    }

    pub fn line_format_from_name(
        &mut self,
        name: &str,
    ) -> Result<Arc<dyn FormatTait + Send + Sync>> {
        if let Some(format) = self.text_formats.get(name) {
            return Ok(format.clone());
        }
        if self.text_order.is_empty() {
            return Err(RegistryNotInitialised.into());
        }
        return Err(RegistryNoSuchFormat {
            fmt: name.to_string(),
        }
        .into());
    }

    /// returns true if we detect this delta operation is a registered block format
    /// Note that this only works for formats that are "\n" for block formats.
    /// So a string operation with value = "hello\nworld" is not recognized as "block format"
    pub fn is_block_fmt(&self, op: &DeltaOperation) -> Result<bool> {
        let new_line = op.insert_value().is_string() && op.insert_value().str_val()?.contains("\n");
        if !new_line {
            return Ok(false);
        }
        for &t in self.block_order.iter() {
            let format: &Arc<dyn FormatTait + Send + Sync> = self.block_formats.get(t).unwrap();
            if format.applies(&op)? {
                return Ok(true);
            }
        }
        if self.text_order.is_empty() {
            return Err(RegistryNotInitialised.into());
        }
        return Ok(false);
    }

    pub fn block_format(&self, op: &DeltaOperation) -> Result<Arc<dyn FormatTait + Send + Sync>> {
        for &t in self.block_order.iter() {
            let format: &Arc<dyn FormatTait + Send + Sync> = self.block_formats.get(t).unwrap();
            if format.applies(op)? {
                return Ok(format.clone());
            }
        }
        if self.text_order.is_empty() {
            return Err(RegistryNotInitialised.into());
        }
        return Err(RegistryNoFormatForOp {
            tpe: "block".to_string(),
            op: op.clone(),
        }
        .into());
    }

    /// returns true if we detect this delta operation is a registered text format
    pub fn line_format(&self, op: &DeltaOperation) -> Result<Arc<dyn FormatTait + Send + Sync>> {
        for t in self.text_order.iter() {
            let format: &Arc<dyn FormatTait + Send + Sync> = self.text_formats.get(t).unwrap();
            if format.applies(op)? {
                return Ok(format.clone());
            }
        }
        if self.text_order.is_empty() {
            return Err(RegistryNotInitialised.into());
        }
        return Err(RegistryNoFormatForOp {
            tpe: "line".to_string(),
            op: op.clone(),
        }
        .into());
    }

    pub fn block_remove_attr(&self, op: &DeltaOperation) -> Result<Attributes> {
        for &t in self.block_order.iter() {
            let format: &Arc<dyn FormatTait + Send + Sync> = self.block_formats.get(t).unwrap();
            if format.applies(op)? {
                return Ok(format.block_remove_attr());
            }
        }
        if self.text_order.is_empty() {
            return Err(RegistryNotInitialised.into());
        }
        return Err(RegistryNoFormatForOp {
            tpe: "block".to_string(),
            op: op.clone(),
        }
        .into());
    }
}

/// The test registry registers only the BASIC formats: text & paragraph.
/// Use by adding this crate as a dev-dependency with feature = test_export enabled.
/// The modules which use this feature, should guard the use xxx with the same
/// `#[cfg()]` as shown below.
#[cfg(any(test, feature = "test_export"))]
static TEST_REGISTRY: OnceLock<Mutex<usize>> = OnceLock::new();
#[cfg(any(test, feature = "test_export"))]
pub fn init_test_registry() {
    TEST_REGISTRY.get_or_init(|| {
        Registry::init_registry();
        let mut r = Registry::get_mut_ref().unwrap();
        r.register_block_fmt(NAME_P_BLOCK, P_FORMAT.deref().clone())
            .unwrap();
        r.register_line_fmt(NAME_TEXT, TEXT_FORMAT.deref().clone())
            .unwrap();
        Mutex::new(1)
    });
}
