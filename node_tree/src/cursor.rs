// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::doc_node::DocumentNode;
use crate::error::Error::{
    AdvanceBeyondEnd, BackspaceBeyondStart, NonEmptyBlockCanNotTraversePrev,
    UnexepectedCursorPosNone,
};
use crate::tree_traverse::{
    get_root, is_doc_root, next_node, next_node_non_zero_length, prev_node,
    prev_node_non_zero_length,
};
use anyhow::Result;
use log::error;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, PartialEq, Default)]
pub enum LocationIdentifyer {
    #[default]
    None,
    After,
    Before,
    At,
}

#[derive(Clone, PartialEq, Default)]
pub enum CursorLocation {
    #[default]
    None,
    After(Arc<DocumentNode>),     // cursor is at the end of this node
    Before(Arc<DocumentNode>),    // cursor is right before this node
    At(Arc<DocumentNode>, usize), // cursor is in this node at position "index"; index < node length
}

impl CursorLocation {
    pub fn doc_node(&self) -> Arc<DocumentNode> {
        match self {
            CursorLocation::At(doc_node, _index) => doc_node.clone(),
            CursorLocation::Before(doc_node) => doc_node.clone(),
            CursorLocation::After(doc_node) => doc_node.clone(),
            CursorLocation::None => {
                panic!("get_doc_node(): cursor position is NONE")
            }
        }
    }

    pub fn index(&self) -> Option<usize> {
        match self {
            CursorLocation::At(_doc_node, index) => Some(*index),
            _ => None,
        }
    }

    pub fn get_location(&self) -> LocationIdentifyer {
        match self {
            CursorLocation::At(_doc_node, _index) => LocationIdentifyer::At,
            CursorLocation::Before(_doc_node) => LocationIdentifyer::Before,
            CursorLocation::After(_doc_node) => LocationIdentifyer::After,
            CursorLocation::None => LocationIdentifyer::None,
        }
    }
}

/// no selection selected if this Cursor::end equals Location::None
#[derive(Clone, PartialEq, Default)]
pub struct Cursor {
    retain: RefCell<usize>,
    start: RefCell<CursorLocation>,
    stop: RefCell<CursorLocation>,
}

///Implementation of collapsed cursor --> single point, no selection
/// - collapsed cursor has start location, but no stop location
impl Cursor {
    pub fn new() -> Self {
        Cursor {
            retain: RefCell::new(0),
            start: RefCell::new(CursorLocation::None),
            stop: RefCell::new(CursorLocation::None),
        }
    }

    /// Set() clones the position of the input cursor
    pub fn from(&self, cursor: &Cursor) {
        *self.retain.borrow_mut() = *cursor.retain.borrow();
        *self.start.borrow_mut() = cursor.start.borrow().clone();
        *self.stop.borrow_mut() = cursor.stop.borrow().clone();
    }

    /// # set_cursor_to_doc_node_edge()
    ///
    /// Helper function to prevent repeating code ...
    ///
    /// The cursor can point to the extremes of the document. In such a case it is
    /// messy what cursor setting to use:
    ///  - last node is a paragraph --> cursor points to `AFTER[last_text]`
    ///  - last node is a empty paragraph --> cursor points to `AT[P]`
    /// Same goes for at the start of the document.
    ///
    /// But also in the middle of a document, when iterating over `Nodes` using `next_node()`
    /// and `prev_node()` we may need to repeatedly check if we are pointing to an empty
    /// paragraph or to some text when setting the cursor
    ///
    /// This function should `NOT` change the retain length of the cursor pointer.
    pub fn set_cursor_to_doc_node_edge(&self, doc_node: &Arc<DocumentNode>, before: bool) {
        assert!(!is_doc_root(doc_node));

        //error!( "set_cursor_to_doc_node_edge - START with cursor before = {}", before);
        //error!( "set_cursor_to_doc_node_edge - START with doc_node = {}", &doc_node);
        if before {
            if doc_node.get_formatter().is_text_format() {
                self.set_before(doc_node);
            } else if let Some(prev) = prev_node_non_zero_length(doc_node) {
                if prev.get_formatter().is_text_format() {
                    self.set_after(&prev);
                } else if doc_node.child_count() == 0 {
                    self.set_at(doc_node, 0); //double block node --> this one is empty
                } else {
                    let next = next_node_non_zero_length(doc_node).unwrap();
                    self.set_before(&next);
                }
            } else {
                //doc_node without previous node must be the first right?
                //since the doc_node is a block format, this block format must be empty
                assert!(get_root(doc_node).get_child(0).unwrap().eq(doc_node));
                self.set_at(doc_node, 0);
            }
        } else if doc_node.get_formatter().is_text_format() {
            self.set_after(doc_node);
        } else if let Some(next) = next_node_non_zero_length(doc_node) {
            if next.get_formatter().is_text_format() {
                self.set_before(&next);
            } else if doc_node.op_len() == 0 {
                self.set_at(&next, 0); //double block node --> this next one is empty
            }
        } else {
            //So there is no next document node. And still we want to go after the last one ...
            // FIXME is this true ... here used to be an error
            //when we are deleting we want to be after the previous one
            //when we are retaining we want to be after the current one
            panic!("Setting cursor after a block node that is the last in the document --> seems wrong");
        }

        //FIXME: Very expensive in case we find the end of document, is there a smarter way?
        *self.retain.borrow_mut() = self.calculate_retain_index();
        //error!( "set_cursor_to_doc_node_edge - END with doc_node = {}", self);
    }

    /// # valid()
    ///
    /// Returns true when the cursor start points to a valid location in the document.<br>
    /// Valid means that the cursor at least points to a location in the document.
    ///
    /// Also:
    /// - both a selection, and a non-selection are valid
    /// - it is not tested if the nodes where the cursor points to are part of the document (may be dangling node ...)
    pub fn valid(&self) -> bool {
        if *self.start.borrow() != CursorLocation::None {
            return true;
        }
        false
    }

    /// S# set_after()
    ///
    /// ets the cursor after the given node.
    pub fn set_after(&self, doc_node: &Arc<DocumentNode>) {
        assert!(doc_node.get_formatter().is_text_format());
        *self.start.borrow_mut() = CursorLocation::After(doc_node.clone());
        //FIXME: Very expensive: op_transform actions know what the index is
        *self.retain.borrow_mut() = self.calculate_retain_index();
    }

    pub fn set_after_no_retain_update(&self, doc_node: &Arc<DocumentNode>) {
        assert!(doc_node.get_formatter().is_text_format());
        *self.start.borrow_mut() = CursorLocation::After(doc_node.clone());
    }

    /// # set_before()
    ///
    /// Sets the cursor before the given node.
    pub fn set_before(&self, doc_node: &Arc<DocumentNode>) {
        assert!(doc_node.get_formatter().is_text_format());
        *self.start.borrow_mut() = CursorLocation::Before(doc_node.clone());
        //FIXME: Very expensive: op_transform actions know what the index is
        *self.retain.borrow_mut() = self.calculate_retain_index();
    }

    pub fn set_before_no_retain_update(&self, doc_node: &Arc<DocumentNode>) {
        if !doc_node.get_formatter().is_text_format() {
            error!("Cursor: wrong location... {:?}", doc_node.get_operation());
            error!("Cursor: wrong location... {}", self);
        }
        assert!(doc_node.get_formatter().is_text_format());
        *self.start.borrow_mut() = CursorLocation::Before(doc_node.clone());
    }

    /// # set_at()
    ///
    /// Sets the cursor at the given node, at the desired index.
    ///
    /// If we are "AT" a block node position 0, then we have to insert the first text block
    /// This only happens when we are appending to the document (includes an empty document)
    pub fn set_at(&self, doc_node: &Arc<DocumentNode>, index: usize) {
        assert!(
            (index > 0 && doc_node.get_formatter().is_text_format()) ||  // index == 0: valid cursor points to "before"
                (index == 0 && !doc_node.get_formatter().is_text_format()) // valid cursor points to "block_node"
        );
        assert!(
            doc_node.op_len() > index ||   //index == op_len(): valid cursor at should point "after" the node
                doc_node.op_len() == 0
        ); //pass block_node: previous assert already checked "OK" for block nodes

        //Finally we do the setting ...
        *self.start.borrow_mut() = CursorLocation::At(doc_node.clone(), index);
        //FIXME: Very expensive: op_transform actions know what the index is
        *self.retain.borrow_mut() = self.calculate_retain_index();
    }

    pub fn set_at_no_retain_update(&self, doc_node: &Arc<DocumentNode>, index: usize) {
        assert!(
            (index > 0 && doc_node.get_formatter().is_text_format()) ||  // index == 0: valid cursor points to "before"
                (index == 0 && !doc_node.get_formatter().is_text_format()) // valid cursor points to "block_node"
        );
        assert!(
            doc_node.op_len() > index ||   //index == op_len(): valid cursor at should point "after" the node
                doc_node.op_len() == 0
        ); //pass block_node: previous assert already checked "OK" for block nodes

        //Finally we do the setting ...
        *self.start.borrow_mut() = CursorLocation::At(doc_node.clone(), index);
    }

    /// # get_location()
    ///
    /// Returns just the start location of the cursor.
    ///
    /// Use get_selection() if both start & stop locations are needed.
    pub fn get_location(&self) -> CursorLocation {
        self.start.borrow().clone()
    }

    pub fn get_doc_node(&self) -> Arc<DocumentNode> {
        self.start.borrow_mut().deref().doc_node()
    }
}

impl Cursor {
    /// # is_selection()
    ///
    /// Returns true, if the document has a valid end point.
    pub fn is_selection(&self) -> bool {
        !matches!(*self.stop.borrow(), CursorLocation::None)
    }

    /// # collapse()
    ///
    /// For now we collapse the end always.
    ///
    /// Future: is there a reason to take the stop position as the new collapsed cursor location?
    pub fn collapse(&self) {
        *self.stop.borrow_mut() = CursorLocation::None;
    }

    /// # selection_length()
    ///
    /// Returns the number of characters between cursor start and stop locations.
    pub fn selection_length(&self) -> usize {
        if !self.is_selection() {
            return 0;
        }

        // When start and stop node is identical we return:
        // - length from start of node until cursor
        // in the other case:
        // - length of the cursor until the end of the node
        // The latter being the complementary value of the first
        let start_is_end_node = self.start.borrow().doc_node() == self.stop.borrow().doc_node();
        let (len, start_doc_node) = match self.start.borrow().clone() {
            CursorLocation::None => {
                return 0;
            }
            CursorLocation::After(doc_node) => (0, doc_node),
            CursorLocation::Before(doc_node) => {
                if start_is_end_node {
                    (0, doc_node)
                } else {
                    (doc_node.op_len(), doc_node)
                }
            }
            CursorLocation::At(doc_node, index) => {
                if start_is_end_node {
                    (index, doc_node)
                } else {
                    (doc_node.op_len() - index, doc_node)
                }
            }
        };

        let (end_len, end_doc_node) = match self.stop.borrow().clone() {
            CursorLocation::None => {
                panic!("No end document node defined. This is not a selection")
            }
            CursorLocation::After(doc_node) => (doc_node.op_len(), doc_node),
            CursorLocation::Before(doc_node) => (0, doc_node),
            CursorLocation::At(doc_node, index) => (index, doc_node),
        };

        //If start node == end node, we should be done after this addition
        let mut length = if start_doc_node != end_doc_node {
            len + end_len
        } else {
            end_len - len
        };

        //Loop until we find the end node, but we do not include the end node (again)
        if start_doc_node != end_doc_node {
            let mut next_o = next_node(&start_doc_node);
            loop {
                if let Some(next) = next_o {
                    if next == end_doc_node {
                        break;
                    } //stop BEFORE adding the end node length!
                    length += next.op_len();
                    next_o = next_node(&next);
                } else {
                    panic!("End of document reached, but end_node not found. This is not a selection, or end < start");
                }
            }
        }

        //Check if we have a valid retain length between start and end that is larger than zero.
        if length == 0 {
            error!("Cursor.rs: Selection length = 0; This is not a selection.");
            panic!("Cursor.rs: {}", &self);
        }
        length
    }

    /// # get_selection()
    ///
    /// Returns start,stop cursor locations
    pub fn get_selection(&self) -> (CursorLocation, CursorLocation) {
        let start = match self.start.borrow_mut().deref() {
            CursorLocation::At(doc_node, index) => CursorLocation::At(doc_node.clone(), *index),
            CursorLocation::Before(doc_node) => CursorLocation::Before(doc_node.clone()),
            CursorLocation::After(doc_node) => CursorLocation::After(doc_node.clone()),
            _ => CursorLocation::None,
        };

        let stop = match self.stop.borrow_mut().deref() {
            CursorLocation::At(doc_node, index) => CursorLocation::At(doc_node.clone(), *index),
            CursorLocation::Before(doc_node) => CursorLocation::Before(doc_node.clone()),
            CursorLocation::After(doc_node) => CursorLocation::After(doc_node.clone()),
            _ => CursorLocation::None,
        };
        (start, stop)
    }

    pub fn set_select_start(&self, location: CursorLocation) {
        self.start.replace(location);
        //FIXME: Very expensive: op_transform actions know what the index is
        if *self.start.borrow() == CursorLocation::None {
            *self.retain.borrow_mut() = 0;
        } else {
            *self.retain.borrow_mut() = self.calculate_retain_index();
        }
    }

    pub fn get_select_start(&self) -> CursorLocation {
        self.start.borrow().clone()
    }

    pub fn set_select_stop(&self, location: CursorLocation) {
        self.stop.replace(location);
    }

    pub fn get_select_stop(&self) -> CursorLocation {
        self.stop.borrow().clone()
    }
}

impl Cursor {
    pub fn get_retain_index(&self) -> usize {
        *self.retain.borrow()
    }

    pub fn set_retain_index(&self, index: usize) {
        *self.retain.borrow_mut() = index;
    }

    /// # calculate_retain_index()
    ///
    /// Returns the retain index number to reach the current cursor location when starting from
    /// the start of the document. Doc_root is not accepted as an input, and will trigger an exception.
    ///
    /// This will re-calculate the retain index Each and Every Time it is called. This may not be
    /// efficient. So consider using get_retain_index() ...
    pub fn calculate_retain_index(&self) -> usize {
        const ZERO: usize = 0;
        let (doc_node, r) = match self.start.borrow_mut().deref() {
            CursorLocation::At(doc_node, index) => (doc_node.clone(), *index),
            CursorLocation::Before(doc_node) => (doc_node.clone(), ZERO),
            CursorLocation::After(doc_node) => (doc_node.clone(), doc_node.op_len()),
            CursorLocation::None => {
                panic!("calculate_retain_index(): cursor position is NONE")
            }
        };
        assert!(!is_doc_root(&doc_node)); //we do not accept root as input

        let mut retain = r;
        let mut dn_o = prev_node(&doc_node);

        loop {
            if let Some(doc_node) = dn_o {
                assert!(!is_doc_root(&doc_node));
                retain += doc_node.op_len();
                dn_o = prev_node(&doc_node);
            } else {
                //We stop when reaching doc_root. I that case prev_node() should return None
                return retain;
            }
        }
    }
}

impl Cursor {
    /// # advance()
    ///
    /// Advances the cursor by just 1 character. No delete of any character
    pub fn advance(&self) -> Result<()> {
        let loc = self.start.borrow_mut().deref().clone();
        match loc {
            CursorLocation::At(doc_node, index) => {
                if doc_node.is_text() {
                    if index + 1 < doc_node.op_len() {
                        self.set_at_no_retain_update(&doc_node, index + 1);
                    } else {
                        self.set_after_no_retain_update(&doc_node);
                    }
                } else {
                    assert_eq!(index, 0);
                    if let Some(next) = next_node_non_zero_length(&doc_node) {
                        if next.is_text() {
                            self.set_before_no_retain_update(&next);
                        } else if let Some(n_next) = next_node_non_zero_length(&next) {
                            if n_next.is_text() {
                                self.set_before_no_retain_update(&n_next);
                            } else {
                                self.set_at_no_retain_update(&next, 0);
                            }
                        }
                    } else {
                        return Err(AdvanceBeyondEnd.into());
                    }
                }
            }
            CursorLocation::After(doc_node) => {
                assert!(doc_node.is_text());
                if let Some(next) = next_node_non_zero_length(&doc_node) {
                    if next.is_text() {
                        self.set_at_no_retain_update(&next, 1);
                    } else if let Some(n_next) = next_node_non_zero_length(&next) {
                        if n_next.is_text() {
                            self.set_before_no_retain_update(&n_next);
                        } else {
                            self.set_at_no_retain_update(&next, 0);
                        }
                    }
                } else {
                    return Err(AdvanceBeyondEnd.into());
                }
            }
            CursorLocation::Before(doc_node) => {
                assert!(doc_node.is_text());
                self.set_at_no_retain_update(&doc_node, 1);
            }
            CursorLocation::None => {
                return Err(UnexepectedCursorPosNone.into());
            }
        };

        let retain = *self.retain.borrow() + 1;
        *self.retain.borrow_mut() = retain;
        Ok(())
    }

    /// # backspace()
    ///
    /// Moves the cursor back by just 1 character. No deletion of any character!!
    pub fn backspace(&self) -> Result<()> {
        let loc = self.start.borrow_mut().deref().clone();
        match loc {
            CursorLocation::At(doc_node, index) => {
                if doc_node.is_text() {
                    if index > 1 {
                        self.set_at_no_retain_update(&doc_node, index - 1);
                    } else {
                        assert_eq!(index, 1); //if the index = 0, we should point "before" the node
                        self.set_before_no_retain_update(&doc_node);
                    }
                } else {
                    assert_eq!(index, 0); //pointing to an empty block format
                    if let Some(prev) = prev_node_non_zero_length(&doc_node) {
                        assert!(!prev.is_text()); //prev of an empty block must be a block
                        if prev.is_empty_block() {
                            self.set_at_no_retain_update(&prev, 0);
                        } else if let Some(p_prev) = prev_node_non_zero_length(&prev) {
                            if p_prev.is_text() {
                                self.set_after_no_retain_update(&p_prev)
                            } else {
                                self.set_after_no_retain_update(&p_prev);
                            }
                        } else {
                            return Err(NonEmptyBlockCanNotTraversePrev.into());
                        }
                    } else {
                        return Err(BackspaceBeyondStart.into());
                    }
                }
            }
            CursorLocation::Before(doc_node) => {
                assert!(doc_node.get_formatter().is_text_format());
                if let Some(prev) = prev_node_non_zero_length(&doc_node) {
                    if prev.is_text() {
                        self.set_at_no_retain_update(&prev, prev.op_len() - 1);
                    } else if let Some(p_prev) = prev_node_non_zero_length(&prev) {
                        if p_prev.is_text() {
                            self.set_after_no_retain_update(&p_prev);
                        } else {
                            // we should not have block in block, so we are an empty block
                            self.set_at_no_retain_update(&prev, 0);
                        }
                    }
                } else {
                    return Err(BackspaceBeyondStart.into());
                }
            }
            CursorLocation::After(doc_node) => {
                assert!(doc_node.get_formatter().is_text_format());
                let len = doc_node.op_len();
                if len > 1 {
                    self.set_at_no_retain_update(&doc_node, doc_node.op_len() - 1);
                } else {
                    self.set_before_no_retain_update(&doc_node);
                }
            }
            CursorLocation::None => {
                return Err(UnexepectedCursorPosNone.into());
            }
        };

        let or = *self.retain.borrow();
        if or > 0 {
            let retain = or - 1;
            *self.retain.borrow_mut() = retain;
        } else {
            // given the above we should NEVER get here !
            return Err(BackspaceBeyondStart.into());
        }
        Ok(())
    }
}

/// Display implementation for the cursor location. This is intended for debugging only.
impl Display for CursorLocation {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            CursorLocation::At(doc_node, index) => {
                let tpe = doc_node.get_doc_dom_node().get_node_name();
                let txt = if doc_node.is_leaf() {
                    doc_node.find_dom_text().get_text()
                } else {
                    "#empty text#".to_string()
                };
                let (left, right) = txt.split_at(*index);
                let txt = [left, "[*]", right].concat();
                write!(
                    f,
                    "Cursor_AT[node type = {}, index = {}, op_len = {}, text = {}]",
                    tpe,
                    index,
                    doc_node.op_len(),
                    txt
                )
            }
            CursorLocation::Before(doc_node) => {
                let tpe = doc_node.get_doc_dom_node().get_node_name();
                let txt = if doc_node.is_leaf() {
                    doc_node.find_dom_text().get_text()
                } else {
                    "#empty text#".to_string()
                };
                let txt = ["[*]", &txt].concat();
                write!(
                    f,
                    "Cursor_Before[node type= {}, op_len = {}, text = {}]",
                    tpe,
                    doc_node.op_len(),
                    txt
                )
            }
            CursorLocation::After(doc_node) => {
                let tpe = doc_node.get_doc_dom_node().get_node_name();
                let txt = if doc_node.is_leaf() {
                    doc_node.find_dom_text().get_text()
                } else {
                    "#empty text#".to_string()
                };
                let txt = [&txt, "[*]"].concat();
                write!(
                    f,
                    "Cursor_After[node type = {}, op_len = {} text = {}]",
                    tpe,
                    doc_node.op_len(),
                    txt
                )
            }
            CursorLocation::None => {
                write!(f, "Cursor_None()")
            }
        }
    }
}

/// Display implementation for the cursor. This is intended for debugging only.
impl Display for Cursor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cursor-->[[\n\tstart = {},\n\tstop = {}\n]]\n",
            self.start.borrow(),
            self.stop.borrow()
        )
    }
}

/// # cursor_points_to()
///
/// Returns the character to which the cursor points.
/// If the cursor does not point to a part of the text, but to a block node,
/// then it will return the HTML tag `<P>` (or which ever tag applies)
///
/// This function allows tests where the location of the cursor has some expected
/// position. In that case we check the character pointed to by the cursor, and
/// compare this with an expected value.
#[cfg(any(test, feature = "test_export"))]
pub fn cursor_points_to(cursor: &Cursor) -> String {
    match cursor.get_location() {
        CursorLocation::None => {
            panic!("hey no location")
        }
        CursorLocation::After(doc_node) => {
            if let Some(next) = next_node(&doc_node) {
                if !next.is_text() {
                    let name = next.get_doc_dom_node().get_node_name();
                    format!("<{name}>")
                } else {
                    next.get_operation()
                        .insert_value()
                        .str_val()
                        .unwrap()
                        .chars()
                        .next()
                        .unwrap()
                        .to_string()
                }
            } else {
                // " --pointing beyond last node -- "
                '!'.to_string()
            }
        }
        CursorLocation::Before(doc_node) => {
            if !doc_node.is_text() {
                let name = doc_node.get_doc_dom_node().get_node_name();
                format!("<{name}>")
            } else {
                doc_node
                    .get_operation()
                    .insert_value()
                    .str_val()
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap()
                    .to_string()
            }
        }
        CursorLocation::At(doc_node, index) => {
            if !doc_node.is_text() {
                let name = doc_node.get_doc_dom_node().get_node_name();
                format!("<{name}>")
            } else {
                doc_node
                    .get_operation()
                    .insert_value()
                    .str_val()
                    .unwrap()
                    .chars()
                    .nth(index)
                    .unwrap()
                    .to_string()
            }
        }
    }
}
