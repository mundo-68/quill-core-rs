# Impact of complex HTML structure on `operational transform` implementation

Some HTML constructs require nested document nodes that are 
created just for display purposes. For example a HTML list, or link.
To allow the operational transform operations to ignore this complexity,
each format_trait shall have 2 functions:
- isolate()
- merge()

After isolation, each DeltaOperation is represented in HTML as its own
HTML "sub-tree". For example in a list:

```html
<UL>
    <LI>item 1</LI>
    <LI>{*}item 2</LI>
    <LI>item 3</LI>
</UL>
```
After calling `isolate()` the `<LI>` items are  in their own "sub-tree":

```html
<UL><LI>item 1</LI></UL>
<UL><LI>{*}item 2</LI></UL>
<UL><LI>item 3</LI></UL>
```

To handle more complex formats, we should `isolate` the current cursor position.
After which the operational transform operations can be executed in a more 
straight forward implementation.

Insert happens `[BEFORE]` the cursor, leaving the cursor in the next
insert position after the split.

 ```html
   [node 1]{*}[node 2][node 3]
 ```

Now we `isolate()` the node that may change from the insert operation, at `node 1`. If it is an
insertion of text, then we add `new content` or if there is an insertion of a block node
we may even change the block node to which `node 1` belongs ....

 ```html
      [node 0][[node 1]]{*}[node 2][node 3]
         --> (insert)
      [node 0][[node 1]][new content]{*}[node 2][node 3]
         ^         ^          ^             ^
         |         |          |             |
  Boundaries:  A         B           C
 ```

And the `merge()` operation should try to merge the nodes depicted at the arrows (pointing up).
Notice that these are 3! boundaries that possibly need to be evaluated. If we start using the format
of `[new content]` then we should evaluate:

- `boundary A`: Check if `[node 0]` and `[node 1]` still match using format `1`<br>
  Normally `[node 0]` and `[node 1]` are not split. so we can ignore this case. But for formats which do split theres ...merge them too!
- `boundary B`: Check if `[node 1]` and `[new content]` match using format `[new content]`
- `boundary C`: Check if `[new content]` and `[node 2]` match using format `[new content]`
  This is all lumped together in a function `try_merge()`

# Insert-operation
`Insert()` starts at the given document node cursor position.
It will insert a new value BEFORE the cursor position, leaving the cursor in the right 
place for the next operation.

# Delete-operation
`Delete(n)` will delete the first `n` characters AFTER the cursor.

# Retain-operation
## Retain without attributes
`Retain(n)` will shift the cursor `n` positions forward
## Retain with attributs
`Retain(n, attr)` will change the attributes for the next `n` characters, and will then leave
the document cursor AFTER the last changed character.

# Soft break

 Tag which belongs to the HTML element `<BR/>` must be empty !! `<br>`
 So we treat this format as a `line` format not a `BLOCK` format.

 We have 2 soft break incarnations:
  - Normal soft break which lives in a DeltaOperation document
  - Automatically inserted (temporary) place holders to display an empty block node

 The automatically inserted place holders shall have empty delta, and are NOT recognized
 when inserted as a normal delta operation:
 ```html
  Insert{
    ""
 }

 The normal page break shows as a character of length 1
 when inserted as a normal delta operation:
 ```bash
  Insert{
    {"page_break", "true"}
 }
 ```

 ## Automatic soft breaks
Some browsers do not display anything for a HTML element that equals:

```bash
<H1></H1>
```

So we need to force the display like a single empty line of the correct format (eg. text size).

One solution is to change the `css` style:

```bash
p:empty{
    height: 1em;
    /*display: none; */
}
```

but that should then be done for ALL formats. Alternative is to add a `<br>` for each empty block,
and remove it again when it is not empty.

Conclusion:<br>
Some browsers have trouble with displaying empty `<p></p>` blocks. So to help these browswers we need
to include a `<br>` to allow the user to set a cursor "in" the paragraph when editing.


