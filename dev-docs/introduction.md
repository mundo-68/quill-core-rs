# Design

This is the `quill-core-rs` core crate provides the basics for an HTML editor.
By design, we have different representations of the same textual information:

- `Delta` document: Linear array of delta operations
- `HTML` document: `DOM` tree, hierarchical structure of nodes.

For more information on the `Delta` format, see the `` crate.<br>
Or this link:
[quill content rendering](https://www.fatalerrors.org/a/content-rendering-mechanism-of-quill-a-modern-rich-text-editor.html)


To change the document content, `quill-core-rs` provides an interface for operational
transforms. `quill-core-rs` will make sure that the internal representations of the 
document `Delta` document, and `HTML DOM` will stay consistent.

The design makes a shadow tree consisting of `document_node` linking both `Delta operations`
and `HTML` `DOM`. These `document_node` contain:

- `Delta` operation
- `DOM-node` (element, or text)
- list of child `document_node`
- reference to parent `DOM-node`

The basic idea is to have an `Delta operation` for each `DOM` node. Since pure text `Delta operation` 
may contain several paragraphs, we split the delta at each `\n`. Hence, we do not make the `Delta` compact 
as is required in the standard `Delta` document definition. That is not a problem, if we keep this intern.

To keep the delta document and the `HTML` `DOM` in sync, we need a method to traverse the `HTML` `DOM` tree
in te same way that the delta document is ordered. For this a `document_node` traversal library is made.
In this library utility functions are created like:

- next_node()
- preve_node()
- next_sibling()
- prev_sibling()
- next_block()
- prev_block()

# Document node and operation types

## `Block` nodes

BLOCKS are used for vertical spacing. Examples:

- paragraphs `<P>`
- Headings `<Hx>`
- Lists `<UL>`/`<OL>`

Block nodes are identified by:

- an attribute
- a default insert value of `\n`

Notice that since `quill-core-rs` splits all `Delta operation` at the "\n" values,
the block nodes have just the insert value of "\n". As a result, the length of 
all `Delta operation` can be determined by measuring the length of the "Insert-value".

## `Line` nodes

Line nodes are horizontally aligned elements, like text, hyperlinks, emoji etc.

- Text `Text` element
- image `<img>`
- link `<a>`

Line nodes are identified by:

- an attribute
- any insert value except the value of `\n`

# Formatting operations
Below is a non-maintained list of `Delta operations`. For a complete list, see the `readme.md` of this crate.

## Line formatting operations

- Background Color - background
- Bold - bold
- Color - color
- Font - font
- Inline Code - code
- Italic - italic
- Link - link
- Size - size
- Strikethrough - strike
- Superscript/Subscript - script
- Underline - underline

## Block formatting operations

- Blockquote - blockquote
- Header - header
- Indent - indent
- List - list
- Text Alignment - align
- Text Direction - direction
- Code Block - code-block

## Block Embeds

- Formula - formula (requires KaTex)
- Image - image
- Video - video
