# Document nodes

 We have an editor; All the editor does is create a linear list of DeltaOperations
 The corresponding DOM structure is a nested list. We need to be able to both edit
 the DOM, and the DeltaOperations.

 To keep the link, we can make a shadow tree linking each dom-node to an DeltaOperation.

 Main links needed between both lists:
   - A HTML node belonging to the Delta-Operation
   - A Delta belonging to a given HTML node

 The first one is easy --> store the link of the Node to the Document structure
 The second is a bit harder: store an event on a HTML node upon selection, that event links
 to the original Document structure element ...

 Since we would like to have a delta operation for each dom node, we split the delta
 valid for each dom node. Hence, we do not make the Delta compact as is required in the
 standard Delta document definition. That is not a problem, if we keep this intern.
 Luckily the Delta implementation will merge operations if possible, so we should be OK-ish.

 See [reference cycles](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)

 Tree transformation functions are functions without "Self".
 They often need the "Self" to be an &Arc<DocumentNode> instead of an &DocumentNode (==&Self).
 These functions are added at the end of this module file, and fall into 3 broad categories:

   1. Parent child relation manipulations
   2. Splitting, Merging of DeltaOperations
   3. Inserting, Deleting in a single DeltaOperation


# Document root node
The document root node is special:
- It is the root of a document node tree
- There is only 1 such root node in a HTML representation of a document

The delta document collects:

  - a document editor state
  - a registry with the supported DeltaOperation types
  - a cursor pointing to some location in the document
  - a root element which points to the HTML DOM node which contains all of the content of this document

 The functions provided on this struct allow for basic control of the document:
  - open / close
  - link to a root HTML DOM node

 Implementation note: The root document node has no parent. All other nodes shall have a parent.
 ```html
 <div class="ql-container ql-snow" id="some_id" >
    <div class="ql-editor" contenteditable="true" >
         <p><br></p>
    </div>
 </div>
 ```

# Document nodes 
FIXME ...

## Virtual document node

Document nodes are structured with virtual nodes which have length `0`. These are needed to support
some of the HTML formatting which requires structure that is not conform lines of text and paragraphs,
but has a deeper nested structure. Note that we handle formats like bold, italic, ... without virtual
nodes.

HTML formats usinig virtual structures:

- lists
- links
- tables

Example block: `<UL><LI>text</LI>...</UL>` --> `<UL>` has length 0

Example line: `<A><EM>link</EM></a>` or `<A><EM>link</EM>more text</a>` --> here `<A>` has length 0

For block nodes, the cursor SHALL NOT point to a virtual **block** node `Cursor::AT[virt, 0]`. That would mean
we have an empty virtual block node. Rule: Virtual block nodes are never empty!
The render plug in modules shall take that responsibility.

### Virtual block node

Virtual block node properties

1. Are handled exclusively in the render modules that generated them;
2. The registry SHALL recognize a virtual block node based on its attributes;
3. A virtual block node has an `INSERT` part of the operation shall be equal to `""`;
4. A virtual block node has at least 1 child

Example:

- block node example set after `<UL><LI>text</LI>[*]</UL>` --> `<UL><LI>text</LI></UL>Cursor::before[next]`
- block node example set before `<UL>[*]<LI>text</LI></UL>` --> `<UL><LI>[*]text</LI></UL>`

### Virtual leaf nodes

Virtual leaf node properties (same as virt block node!!):

1. Are handled exclusively in the render modules that generated them;
2. The registry SHALL recognize a virtual block node based on its attributes
3. A virtual leaf node has an `INSERT` part of the operation shall be equal to `""`.
4. A vurtual leaf node has at least 1 text character length

So how do we recognize make the difference between a virtual leaf and block node?
Well --> the render plugin module "sais-so" when asked.
That said: the leaf nodes can be embedded in other block nodes; Block nodes can normally not,
unless we start looking at tables.

Example:

- line node example set after `<A><EM>link</EM>[*]</a>` --> `<A><EM>link</EM></a>before[next]`
- line node example set before `<A>[*]<EM>link</EM></a>` --> `<A>[*]<EM>link</EM></a>`

Notice: that we keep the cursor behaviour identical for both the virtual block, and leaf nodes!

# The document node tree traversal
## Forward traversal

Iterator methods to traverse the tree in a LINEAR order of the document-transformations
The doc-node tree may look like below. Since we have the "block" operations AFTER the
declaration of the leaf node, we should return the block B after E, which comes after D.
 ```text
       A
      / \
     B   C
    / \ / \
   D  E F  G
```
Repeated calls to `Next()` should return :D, E, B, F, G, C, A

Debug hit: If there are `*.unwrap()` errors from this library, then it is probably a
`unlinked-node` that we use as input.

Maybe have a look at the traversal package:
[DftPost](https://docs.rs/traversal/0.1.2/traversal/struct.DftPost.html]) (Depth-First Traversal in Post-Order)

## Backward traversal

Backwards traversal returns the previous document node.

Iterator methods to traverse the tree in a backwards LINEAR order of the document-transformations
 ```text
      A
     / \
    B   C
   / \ / \
  D  E F  G
 ```
Repeated calls to `Prev()` should return: (A), C, G, F, B, E, D

1) Keep going to the last child, until that child is not a leaf anymore
2) If you are not the last sibling, go to the previous sibling
3) If you are the first sibling, and the parent == root --> you are done
4) if not 3) then, go to the parent-parent previous sibling, and try 2) and 3) again

Moves from node to node, depth first as shown in the header text of this file

## Convenience functionality

Sometimes we are only loolking for `line` operations:
- prev_leaf()
- next_leaf()

Sometimes we are only looking for `block` operations:
- prev_block()
- next_block()

# **FIXME: resolve that nasty business with virtual document nodes.**