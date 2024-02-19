# Quill-core-rs
A `Rust` re-implementation of the well known Quill HTML editor.

The functionality provided by this package is:
 - Translate a `Delta` document to a HTML `DOM`
 - Edit the Delta document using operational transform commands
 - Translate the current `DOM` state back to a `Delta` document
 - Provide a set of formats that translate `Delta operations` to HTML `DOM`

Documentation:
 - user docs: https://mundo-68.github.io/quill-core-rs/
 - design docs; see `dev-docs` folder

# Todo

  - [ ] clean up code and simplify
  - [ ] automate wasm-pack tests (only run per crate now)
  - [ ] see missing formats below


# Usage

```rust
fn main() {
    DocumentRoot::set_log_level(Level::Debug);
    register_all();
    //Assuming there is a <div id="some_id"></div> 
    let mut doc = DocumentRoot::new("some_id");
    doc.open();
    //fetch some delta document
    doc.apply_dela(delta);
}

fn register_all() -> Result<()> {
    let register = Registry::get_mut_ref()?;
    register.register_block(NAME_OL_BLOCK, Arc::new(ListBlock::new_ol()));
    register.register_block(NAME_UL_BLOCK, Arc::new(ListBlock::new_ul()));
    register.register_block(NAME_HEADER, Arc::new(HeaderBlock::new()));
    register.register_block(NAME_CODE, Arc::new(CodeBlock::new()));
    register.register_block(NAME_P_BLOCK, Arc::new(Pblock::new()));
    register.register_text(NAME_LINK, Arc::new(LinkFormat::new()));
    register.register_text(NAME_IMAGE, Arc::new(ImageFormat::new()));
    register.register_text(NAME_SOFT_BREAK, Arc::new(SoftBreak::new()));
    register.register_text(NAME_TEXT, Arc::new(TextFormat::new()));
    Ok(())
}
```

# Supported formats
## `Line` formatting operations
`Line` nodes are horizontally aligned elements.

Font related:
- [x] Background Color
- [x] Bold 
- [x] Font Color
- [x] Font 
- [ ] Inline Code 
- [x] Italic 
- [x] Link 
- [x] Size 
- [x] Strikethrough 
- [x] Superscript/Subscript 
- [x] Underline

Other:
- [x] Inline image `<img>`
  - [ ] emoji
- [x] link `<a>`

## `Block` formatting operations
`Block` nodes are used for vertical formatting.

- [x] paragraphs `<P>`
- [x] Headings `<Hx>`
  - [ ] automatic header numbering
- [x] Lists 
  - [x] ordered
  - [x] bullet
  - [ ] list headers with automatic numbering
- [x] Code
- [ ] Tables
  - [ ] table headers with automatic numbering
- [ ] Nicely formatted alert / warn / error blocks
- [ ]Blockquote - blockquote
- [x] Header - header
- [x] Indent - indent
- [x] List - list
- [x] Text Alignment - align
- [x] Text Direction - direction
- [x] Code Block - code-block
- [ ] Image - image
  - [ ] automated image footer and numbering
  
## Block Embeds
- [ ] Formula - formula ([Retex](https://github.com/ReTeX/ReX))
- [ ] Video - video


## License

Licensed under either of
* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## WASM testing

For WASM testing use:

```text
wasm-pack test --headless --chrome --test document
wasm-pack test --headless --firefox
```

To run the other rust tests use:

```text
cargo test
```

