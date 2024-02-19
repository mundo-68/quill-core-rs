#[cfg(test)]
mod test {
    use anyhow::Result;
    use core_formats::paragraph::Pblock;
    use core_formats::text_formatter::TextFormat;
    use delta::attributes::Attributes;
    use delta::delta::Document;
    use delta::operations::DeltaOperation;
    use node_tree::cursor::cursor_points_to;
    use node_tree::dom_doc_tree_morph::append;
    use node_tree::format_trait::FormatTait;
    use op_transform::doc_root::DocumentRoot;
    use std::sync::Arc;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// FIXME: Move to op_transform package
    ///
    ///
    ///
    fn create_text(doc: &DocumentRoot) -> Result<()> {
        let p_format = Arc::new(Pblock::new());
        let t_format = Arc::new(TextFormat::new());

        let mut attr = Attributes::default();
        attr.insert("bold", true);

        let root = doc.get_root();

        let delta = DeltaOperation::insert("\n");
        let par = p_format.create(delta, p_format.clone())?;
        append(&root, par.clone());

        let delta = DeltaOperation::insert("TEXT_1_1");
        let t = t_format.create(delta, t_format.clone())?;
        append(&par, t.clone());

        let delta = DeltaOperation::insert_attr("TEXT_1_2", attr.clone());
        let t = t_format.create(delta, t_format.clone())?;
        append(&par, t.clone());

        let delta = DeltaOperation::insert("TEXT_1_3");
        let t = t_format.create(delta, t_format.clone())?;
        append(&par, t.clone());

        let delta = DeltaOperation::insert("\n");
        let par = p_format.create(delta, p_format.clone())?;
        append(&root, par.clone());

        let delta = DeltaOperation::insert("TEXT_2_1");
        let t = t_format.create(delta, t_format.clone())?;
        append(&par, t.clone());

        let delta = DeltaOperation::insert_attr("TEXT_2_2", attr.clone());
        let t = t_format.create(delta, t_format.clone())?;
        append(&par, t.clone());

        let delta = DeltaOperation::insert("\n");
        let par = p_format.create(delta, p_format.clone())?;
        append(&root, par.clone());

        let expect = r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
        assert_eq!(doc.as_html_string(), expect);
        Ok(())
    }

    #[wasm_bindgen_test]
    fn cursor_advance_test() -> Result<()> {
        let doc = DocumentRoot::new("cursor_advance_test");
        doc.append_to_body();
        create_text(&doc)?;
        doc.reset_cursor();

        assert_eq!(doc.to_delta().document_length(), 43);
        assert_eq!(doc.get_cursor().get_retain_index(), 0);
        //-------------------------------------------------------------------
        // r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
        let s = "TEXT_1_1TEXT_1_2TEXT_1_3#TEXT_2_1TEXT_2_2##".to_string();
        for i in 0 as usize..doc.to_delta().document_length() {
            let cursor = doc.get_cursor();
            let c = s.chars().nth(i).unwrap();
            if c == '#' {
                assert_eq!(cursor_points_to(cursor), "<P>");
            } else {
                let cursor_c = cursor_points_to(&cursor);
                assert_eq!(cursor_c, c.to_string());
            }
            cursor.advance()?;
        }

        Ok(())
    }

    /// We index over the document length -1.<br>
    /// When the cursor goes to the end of the document
    /// it goes to BEFORE the last "\n". Hence -1.
    #[wasm_bindgen_test]
    fn cursor_backspace_test() -> Result<()> {
        let doc = DocumentRoot::new("cursor_backspace_test");
        doc.append_to_body();
        create_text(&doc)?;
        doc.cursor_to_end();

        assert_eq!(doc.to_delta().document_length(), 43);
        assert_eq!(doc.get_cursor().get_retain_index(), 42);
        //-------------------------------------------------------------------
        // r#"<p>TEXT_1_1<strong>TEXT_1_2</strong>TEXT_1_3</p><p>TEXT_2_1<strong>TEXT_2_2</strong></p><p></p>"#;
        let s = "TEXT_1_1TEXT_1_2TEXT_1_3#TEXT_2_1TEXT_2_2##".to_string();
        for i in (1 as usize..doc.to_delta().document_length()).rev() {
            let cursor = doc.get_cursor();
            let c = s.chars().nth(i).unwrap();
            if c == '#' {
                assert_eq!(cursor_points_to(cursor), "<P>");
            } else {
                assert_eq!(cursor_points_to(cursor), c.to_string());
            }
            cursor.backspace()?;
        }
        Ok(())
    }
}
