// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::cursor::{Cursor, CursorLocation};
use crate::doc_node::DocumentNode;
use crate::dom_doc_node::{find_doc_node_from_element_node, find_doc_node_from_text_node};
use dom::dom_text::DomText;
use log::debug;
use std::sync::Arc;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Document, Node, Range, Selection, Window};

/// # DomCursor
///
/// Struct with operations to allow user input from the HTML document.
///
/// When the user clicks, or selects part of the HTML DOM, then this
/// selection must be translated to a cursor in the DocumentNode-tree.
/// This module will help do this.
///
/// The RUST WASM interface to the browser javacript interfaces are used to
/// realise this:
/// [javascript selection range](https://javascript.info/selection-range)
///
/// FIXME: Move this to the editor client ?
#[derive(Clone)]
pub struct DomCursor {
    window: Window,               //helper data
    document: Document,           //helper data
    root_node: Arc<DocumentNode>, //the document being selected in ...
}

impl DomCursor {
    pub fn new(root_node: &Arc<DocumentNode>) -> Self {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        DomCursor {
            window,
            document,
            root_node: root_node.clone(),
        }
    }

    /// # location_in_selection()
    ///
    ///
    /// Returns true if the given location is in the current HTML DOM selection range.
    ///
    /// Note there is a tricky part with `Before` and `After` cursor locations.
    /// For the user these seem the same, but the associated doc node may be right
    /// at the edge of the selection and hence may just be "in" or "out".
    /// We do not check for this kind of edge cases in this function.
    pub fn location_in_selection(&self, location: CursorLocation) -> bool {
        match location {
            CursorLocation::None => false,
            CursorLocation::After(doc_node) => {
                self.is_in_selection(doc_node.get_html_node(), doc_node.op_len())
            }
            CursorLocation::Before(doc_node) => self.is_in_selection(doc_node.get_html_node(), 0),
            CursorLocation::At(doc_node, index) => {
                self.is_in_selection(doc_node.get_html_node(), index)
            }
        }
    }

    /// #is_in_selection()
    ///
    /// Returns true if the given node is in the current HTML DOM selection range.
    pub fn is_in_selection(&self, node: &Node, index: usize) -> bool {
        // Get selection
        let selection = self.fetch_selection();

        // Get the current selection range. Create a new range if necessary.
        let range = if selection.range_count() == 0 {
            return false;
        } else {
            selection
                .get_range_at(0)
                .expect("Could not get range at index 0")
        };
        range
            .is_point_in_range(node, index as u32)
            .expect("failed to check range")
    }

    /// # cursor_from_html_dom()
    ///
    /// Retrieves the dom cursor and sets the document node cursor to the same location
    pub fn cursor_from_html_dom(&self) -> Cursor {
        let cursor = Cursor::new();

        // Get selection
        let selection = self.fetch_selection();

        // Get the current selection range. Create a new range if necessary.
        let count = selection.range_count();
        let range = if count == 0 {
            cursor.set_select_start(CursorLocation::None);
            cursor.set_select_stop(CursorLocation::None);
            return cursor;
        } else if count == 1 {
            selection
                .get_range_at(0)
                .expect("Could not get range at index 0")
        } else {
            panic!("DomCursor::get_selection() - We have multiple selections. Dont know what to do with that.")
        };

        let start_offset = range.start_offset().expect("Could not get start offset") as usize;
        let start_container = range
            .start_container()
            .expect("Could not get start container");
        cursor.set_select_start(self.get_location(start_container, start_offset));

        if !range.collapsed() {
            let end_offset = range.end_offset().expect("Could not get end offset") as usize;
            let end_container = range.end_container().expect("Could not get end container");
            cursor.set_select_stop(self.get_location(end_container, end_offset));
        } else {
            cursor.set_select_stop(CursorLocation::None);
        }
        cursor
    }

    /// # get_location()
    ///
    /// Internal function to get a CursorLocation<br>
    ///
    /// Retrieves the DocumentNode (in the shape of a CursorLocation) from a given
    /// HTML Node as input.
    fn get_location(&self, start_container: Node, start_offset: usize) -> CursorLocation {
        //let mut location = CursorLocation::None;

        //let html = dom_print::pretty_print(&start_container, true );
        //error!( "Clicked on node at offset = {} ...{}", start_offset, html);
        return match start_container.node_type() {
            Node::TEXT_NODE => {
                let doc_node_o = find_doc_node_from_text_node(&start_container, &self.root_node);
                if let Some(doc_node) = doc_node_o {
                    let node_len = doc_node.get_operation().op_len();
                    if start_offset == 0 {
                        CursorLocation::Before(doc_node)
                    } else if start_offset == node_len {
                        CursorLocation::After(doc_node)
                    } else if start_offset < node_len {
                        CursorLocation::At(doc_node, start_offset)
                    } else {
                        CursorLocation::None
                    }
                } else {
                    let text = DomText::from(start_container);
                    let parent = text.get_parent().unwrap();
                    let doc_node_o = find_doc_node_from_text_node(parent.as_ref(), &self.root_node);
                    if let Some(doc_node) = doc_node_o {
                        let node_len = doc_node.get_operation().op_len();
                        if start_offset == 0 {
                            CursorLocation::Before(doc_node)
                        } else if start_offset == node_len {
                            CursorLocation::After(doc_node)
                        } else if start_offset < node_len {
                            CursorLocation::At(doc_node, start_offset)
                        } else {
                            debug!(
                                "Doc Node operation: {:?}\n Doc Node text: {:?}",
                                doc_node.get_operation(),
                                text.get_text()
                            );
                            panic!( "PANIC: DomCursor::get_selection(): Hey it seems the browser does not match the document operation node.");
                        }
                    } else {
                        panic!("DomCursor::get_selection(): Node is a text node, but we have not found one ... programming error? ")
                    }
                }
            }
            Node::ELEMENT_NODE => {
                if start_offset == 0 {
                    let node = if start_container.node_name() == "BR" {
                        start_container.parent_node().unwrap()
                    } else {
                        start_container
                    };
                    let doc_node = find_doc_node_from_element_node(&node, &self.root_node).unwrap();
                    CursorLocation::At(doc_node, 0)
                } else if let Some(prev_sibling) =
                    start_container.child_nodes().get(start_offset as u32 - 1)
                {
                    if prev_sibling.node_type() == Node::TEXT_NODE {
                        let doc_node =
                            find_doc_node_from_text_node(&start_container, &self.root_node);
                        CursorLocation::After(doc_node.unwrap_throw())
                    } else {
                        panic!(
                            "PANIC: DomCursor::get_selection(): Prev node is an ELEMENT.{} ",
                            start_container.node_name()
                        );
                    }
                } else {
                    panic!("PANIC: DomCursor::get_selection(): Strange ... why am I here?.");
                }
            }
            _ => {
                panic!( "PANIC: DomCursor::get_selection(): Not text, and not an element!! (header anyone??).");
            }
        };
    }

    /// # cursor_to_html_dom()
    ///
    /// Sets the dom cursor to the same location pointed to by the DocumentNode Cursor
    /// ```javascript
    /// resetExample() {
    ///       p.innerHTML = `Example: <i>italic</i> and <b>bold</b>`;
    ///       result.innerHTML = "";
    ///
    ///       range.setStart(p.firstChild, 2);
    ///       range.setEnd(p.querySelector('b').firstChild, 3);
    ///
    ///       window.getSelection().removeAllRanges();
    ///       window.getSelection().addRange(range);
    ///     }
    /// ```
    pub fn cursor_to_html_dom(&self, cursor: &Cursor) {
        //error!("DomCursor::set_selection() {}", &cursor);

        let selection = self.fetch_selection();
        selection.remove_all_ranges().unwrap_throw();
        let range = self.create_range();

        //Make sure we remove the selection, and just have 1 pointer.
        //If we are a selection, we set the end point below
        //range.collapse_with_to_start(true);

        // Set range start
        match cursor.get_select_start() {
            CursorLocation::After(doc_node) => {
                range
                    .set_start_after(doc_node.find_dom_text().node())
                    .expect("Could not set_start_after");
            }
            CursorLocation::Before(doc_node) => {
                range
                    .set_start_before(doc_node.find_dom_text().node())
                    .expect("Could not set_start_before");
            }
            CursorLocation::At(doc_node, index) => {
                if doc_node.get_formatter().is_text_format() {
                    range
                        .set_start(doc_node.find_dom_text().node(), index as u32)
                        .expect("Could not set_start_at");
                } else {
                    //FIXME: Safari has a bug in the browser
                    //FIXME  that does not allow the cursor to be set at an empty element; but we always have <BR>
                    range
                        .set_start(doc_node.get_html_node(), 0_u32)
                        .expect("Could not set_start_at");
                }
            }
            CursorLocation::None => {
                debug!("setting start selection, but no valid cursor position found.");
            }
        }

        // Set range end
        match cursor.get_select_stop() {
            CursorLocation::Before(doc_node) => {
                range
                    .set_end_before(doc_node.find_dom_text().node())
                    .expect("Could not set_end_before");
            }
            CursorLocation::After(doc_node) => {
                range
                    .set_end_after(doc_node.find_dom_text().node())
                    .expect("Could not set_end_after");
            }
            CursorLocation::At(doc_node, index) => {
                if doc_node.is_leaf() {
                    range
                        .set_end(doc_node.find_dom_text().node(), index as u32)
                        .expect("Could not end_at");
                } else {
                    range
                        .set_end(doc_node.get_html_node(), 0_u32)
                        .expect("Could not end_at");
                }
            }
            CursorLocation::None => {
                //Nothing to do, range is already collapsed
            }
        }

        //Repaired bug here: Add range to selection AFTER setting the range to the proper nodes !!
        selection
            .add_range(&range)
            .expect("failed to add range to selection");
    }

    /// # fetch_selection()
    ///
    /// Return the DOM selection.
    fn fetch_selection(&self) -> Selection {
        match self
            .window
            .get_selection()
            .expect("Could not get selection from window")
        {
            Some(sel) => sel,
            None => {
                panic!("Could not get window selection");
            }
        }
    }

    /// # create_range()
    ///
    /// Creates a HTML DOM range
    fn create_range(&self) -> Range {
        self.document
            .create_range()
            .expect("Could not create range")
    }
}

// /// Activate the specified selection range in the DOM. Remove all previous
// /// ranges.
// fn activate_selection_range(selection: &Selection, range: &Range) {
//     // Note: In theory we don't need to re-add the range to the document if
//     //       it's already there. Unfortunately, Safari is not spec-compliant
//     //       and returns a copy of the range instead of a reference when using
//     //       selection.getRangeAt(). Thus, we need to remove the existing
//     //       ranges and (re-)add our range to the DOM.
//     //
//     //       See https://bugs.webkit.org/show_bug.cgi?id=145212
//     selection.remove_all_ranges().expect("Could not remove all ranges");
//     selection.add_range(&range).expect("Could not add range");
// }

//
// #[cfg(tests)]
// mod tests {
//     #[tests]
//     fn tests() {
//         assert!(true);
//     }
// }
