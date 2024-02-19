# Document cursor
There are 2 cursors types that are relevant to `quill-core-rs`:
 - a cursor pointing to document nodes.
 - a cursor pointing to DOM nodes

The document cursor is relevant for handling delta operations. These can be executed either from the 
document start, and from the current location of the cursor. 

The dom node cursor is good to show the user the current document position when editing a graphical
HTML document. Or when a user changes the cursor position using eg. a mouse click to find the 
corresponding document node cursor.

## Cursor locations

Cursor position: requirements for a valid cursor position:

1. A cursor ALWAYS points to a leaf node
2. Exception An empty Block node is pointed to by `Cursor::AT[block_node, index=0]`
3. Insertion will always happen _BEFORE_ the current cursor location, doing so leaves the cursor just right for the next operation
4. Deletion will always happen _AFTER_ the current cursor location, doing so leaves the cursor just right for the next operation
5. Cursor **SHALL NEVER** be **AFTER** the last `<P>` block which is the last block of a document by definition

Example:`<TEXT>helo world</TEXT>`&& cursor is at `hel[*]o world`

- `Delete(1)` renders; `<TEXT>hel world</TEXT>`
- `Insert("l")` renders; `<TEXT>hello world</TEXT>`

Example of cursor positions for normal `line` and `block` nodes:

| CURSOR                                                | HTML                                                                                                                                            |
| ----------------------------------------------------- |-------------------------------------------------------------------------------------------------------------------------------------------------|
| cursor AT a position                                  | `<P><EM>bold</EM>te[*]xt</P><P></P>`                                                                                                            |
| cursor AT a (empty) block node position               | `<P><EM>bold</EM>text</P><P>[*]</P>`                                                                                                            |
| cursor BEFORE a position (text)                       | `<P><EM>bold</EM>[ *]text</P><P></P>`                                                                                                           |
| semantically identical cursor AFTER a position (bold) | `<P><EM>bold</EM>[* ]text</P><P></P>`                                                                                                           |
| cursor AFTER a position (text)                        | `<P><EM>bold</EM>text[*]</P><P></P>`                                                                                                            |
| cursor `invalid` position (should be AT[P])           | Invalid `<P><EM>bold</EM>text</P>[*]<P></P>`<br/> valid: `<P><EM>bold</EM>text</P><P>[*]</P>` <br/> valid: `<P><EM>bold</EM>text[*]</P><P></P>` |


## Virtual document nodes
 `DocumentNode` trees contain virtual nodes which have length 0, and
 which have no (or empty) DeltaOperation attached to it. This construct
 is needed because the HTML DOM-tree has sometimes nested DOM nodes which we can not 1:1
 link to a "DocumentNode" which has a non-empty DeltaOperation. We chose to
 have the "leaf" nodes attached to the DeltaOperation, and the container node
 empty. In this way the DomTree is 1:1 linked to the DocumentNodeTree.

See the `document_node.md` document for more details.

### Example requiring a virtual node for a block format

The HTML list: 
```html
   <UL><LI>text</LI>...</UL>
```

  - The `<LI>` linked to a document(block)node is child of the `<UL>` DOM node.
  - The `<UL>` document(virtual)node has length 0, and no/empty DeltaOperation


### Example requiring a virtual node for a line  format

The HTML link: 
```html
    <A><EM>link</EM></a> <A><EM>link</EM>more text</a>
```

  - The `<EM>` document(block)node is child of the `<A>` DOM node.
  - The `<A>` document(virtual)node has length 0, and no/empty DeltaOperation

### A cursor *never* points to a virtual document node

 The cursor SHALL NOT point to a node with length = 0 (virtual document node). These nodes are
 handled exclusively by implementations of  `DocumentNode:::FormatTrait` implementations;
 which are also responsible to generated them.
 The ```editor::core::registry``` SHALL recognize them with the correct render module based
 on its attributes, but the "INSERT" part of the operation shall be equal to `""`.

 Now what does this mean for the cursor module. Well we check here if we have such a document node
 and make sure we never point to such virtual doc node. This allows great simplification of all other code

#### block node example for a list pointing "after" the last list item `<li>`:
Set cursor after: `<UL><LI>text</LI>{*}</UL><next ...>` <br>
Will put the cursor: `<UL><LI>text</LI></UL>before<next ...>`

#### line node example for a link pointing "after" the last item in the link `<a>`:
Set cursor after: `<A><EM>link</EM>{*}</a><next ...>` <br>
Will put the cursor: `<A><EM>link</EM></a>before<next ...>`

Note: ONLY if we are the last child of the virtual parent, we need to move the cursor

#### block node example for a list pointing "before" the first list item:
Set cursor before `<P></P><UL>{*}<LI>text</LI></UL>` <br>
Will put the cursor: `<P>{At(*)}</P><UL><LI>text</LI></UL>`

#### line node example for a link pointing "before" the last item in the link `<a>`:
Set cursor before `<before...><A><EM>link</EM>{*}</a>` <br>
Will put the cursor: `<before...>{after(*)}<A><EM>link</EM></a>`

Note that we have to check the PARENT for op_len() && this node being the first of the parent !!


## Cursor setting of virtual nodes

### Block nodes

Example: `<UL><LI>text</LI><LI>text</LI></UL><P.</P>`

```text
cursor[A]
1 - { Insert: "text", Attributes {}},
2 - { Insert: "\n", Attributes {list:bullet}}
3 - { Insert: "text", Attributes {}}
cursor[B]
4 - { Insert: "\n", Attributes {list:bullet}}
cursor[C]
5 - { Insert: "\n", Attributes {}}
```

Document node structure with above cursor positions in the tree

```text
             root                               root...... after block_split() at te[*]xt of first list element
             / \                                /      / \
v-block    UL   P(C)  real block               UL(R)  UL  P    (R) is returned left hand side node of block_split()
          /  \                                 /     /  \
         LI  LI(X)    real block              LI   LI  LI   
        /    /                                /    /   / 
   (A)text  text(B)                          te  xt  text
```

If we create a new `LI` node we get the `UL` node back to insert. Then according to the rules,
a `try_merge()` call will be done to clean up the resulting tree. Hence if possible two
consequtive `UL` blocks are merged into one. Note that we can not put the cursore aftr the last LI node; it
will shift to the AT[P] which is the next node.

**Retain rules:**

- Retain will call `next_node()` if a virtual node is encountered until the first non virtual is found

**Insert rules:**

- `Op_insert()`
 - Inserting a **leaf** node: `Split_leaf()` `AT` some location, then insert at the cursor position.
 - Inserting a **block** node: `Split_block()` which returns the doc_node to transform.
   Example: `List` will split <UL> node before adding anything
- `Render::create(delta operation)` returns the root of some sub-structure created
- `Render::merge(doc_node, before, cursor)` will update the doc-tree, and the cursor location.
  We will have to input a doc_node, since a cursor shall NOT point to a virtual block node.
  If we would give `cursor[next(node)]` to allow us to pass only the cursor to the render function,
  then `render(cursor)` may not find the proper nodes to merge.

**DELETE rules:**

- If a virtual node is found, it is skipped
- The last node at which we stop is NOT a virtual (block) node

### Line nodes

Example: `<P><A><EM>bold</EM>text</A></P>`

```text
cursor[A]
1 - { Insert: "bold", Attributes {link:http:xx; bold:true}},
cursor[B]
2 - { Insert: "text", Attributes {link:http:xx; }}
cursor[C]
3 - { Insert: "\n", }
cursor[D]
4 - { Insert: "\n", }
```

Document node structure with above cursor positions in the tree (ignoring that bold is also an element)

```text
            root
            / \
          P   P(D)
         / 
        A(C)    virtual leaf
      /  \   
(A)bold  (B)text
```

#### Retain rules:

- Retain will call `next_node()` if a virtual node is encountered until the first non virtual is found

#### Insert rules:

- `Op_insert()`
 - Inserting a **leaf** node: `Split_leaf()` `AT` some location, then insert at the cursor position.
 - Inserting a **block** node: `Split_block()` which returns the doc_node to transform.
- `Render::create(delta operation)` returns the root of some sub-structure created
- `Render::merge(doc_node, before, cursor)` will update the doc-tree, and the cursor location.
  We will have to input a doc_node, since a cursor shall NOT point to a virtual block node.
  Further if we give `cursor[next(node)]` to allow us to pass only the cursor to the render function,
  then `render(cursor)` may not find the proper nodes to merge.

####  DELETE rules:

- if a virtual node is found, it is skipped
- the last node at which we stop is NOT a virtual (block) node


# DOM Cursor

## Set the DOM cursor to a given location pointed to by the document node cursor

To do so, we find the HTML node pointed to by the cursor, and then set the HTML range.

 Sets the dom cursor to the same location pointed to by the DocumentNode Cursor
 ```javascript
 resetExample() {
       p.innerHTML = `Example: <i>italic</i> and <b>bold</b>`;
       result.innerHTML = "";

       range.setStart(p.firstChild, 2);
       range.setEnd(p.querySelector('b').firstChild, 3);

       window.getSelection().removeAllRanges();
       window.getSelection().addRange(range);
     }
 ```

## Get the document node cursor FROM a given location pointed to by the DOM cursor

Here we loop over all document nodes, in oder to find the DOM node that is associated with the 
HTML range node.