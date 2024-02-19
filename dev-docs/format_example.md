
# Examples of `Delta` documents, and their HTML representation

## Inline text 1

```
The text:
(Darn no underline in Markdown...)
Hello **sweet** little world.<BR>
This has been a pleasure<BR>
But why did we __underline__ this word?<BR>
I would like ++italics and underline++ better!<BR>
<BR>
well...maybe
```

### Formats as Delta document

```json
{
  "ops": [
    { "insert": "Hello " },
    { "attributes": { "bold": true }, "insert": "sweet" },
    { "insert": " little world. \nThis has been a pleasure \nBut why did we " },
    { "attributes": { "underline": true }, "insert": "underline" },
    { "insert": " this word?\nI would like " },
    {
      "attributes": { "underline": true, "italic": true },
      "insert": "italics and underline"
    },
    { "insert": " better!\n\nwell...maybe\n" }
  ]
}
```

## Inline text 2

### The text

```
This text is red background  bold and bold-italic.<br>
This text has font1 and this one another font
```

### Formats as Delta document

```json
{
  "ops": [
    { "insert": "This text is " },
    { "attributes": { "color": "#e60000" }, "insert": "red" },
    { "insert": " " },
    { "attributes": { "background": "#e60000" }, "insert": "background" },
    { "insert": " " },
    { "attributes": { "bold": "true" }, "insert": "bold" },
    { "insert": " and " },
    {
      "attributes": { "italic": "true", "bold": "true" },
      "insert": "bold-italic."
    },
    { "insert": "\nThis text has " },
    { "attributes": { "font": "serif" }, "insert": "font1" },
    { "insert": " and this one " },
    { "attributes": { "font": "monospace" }, "insert": "another font" },
    { "insert": "\n" }
  ]
}
```

### And as HTML

```html
<div class="ql-container ql-snow" id="some_id" >
    <div class="ql-editor" contenteditable="true" >
    <p>
        This text is 
        <span style="color: rgb(230, 0, 0);">red</span> 
        <span style="background-color: rgb(230, 0, 0);">background</span>  
        <strong>bold</strong> 
        and
        <strong><em>bold-italic.</em></strong>
    </p>
    <p>
        This text has 
        <span class="ql-font-serif">font1</span>
         and this one 
        <span class="ql-font-monospace">another font</span>
    </p>
    </div>
</div>
```

## Block - List

### The text:

```
Hello **sweet** little world.<br>
This has been a pleasure 
  * 1st line
    * 2nd line
    * 3rd line

well...maybe
```

### Formats as Delta document

```json
{
  "ops": [
    { "insert": "Hello " },
    { "attributes": { "bold": true }, "insert": "sweet" },
    { "insert": " little world. \nThis has been a pleasure \n1st line" },
    { "attributes": { "list": "bullet" }, "insert": "\n" },
    { "insert": "2nd line" },
    { "attributes": { "list": "bullet" }, "insert": "\n" },
    { "insert": "3rd line" },
    { "attributes": { "list": "bullet" }, "insert": "\n" },
    { "insert": "\nwell...maybe\n" }
  ]
}
```

## centring

### The text

```
A normal line<br>
--------------a centred line<br>
A normal line<br>
<br>
A normal line<br>
--------------a centred line<br>
--------------and another one<br>
A normal line<br>
```

### Formats as Delta document

```json
{
  "ops": [
    { "insert": "A normal line\na centred line" },
    { "attributes": { "align": "center" }, "insert": "\n" },
    { "insert": "A normal line\n\nA normal line\na centred line" },
    { "attributes": { "align": "center" }, "insert": "\n" },
    { "insert": "and another one" },
    { "attributes": { "align": "center" }, "insert": "\n" },
    { "insert": "A normal line↵" }
  ]
}
```

### with HTML

```html
<div class="ql-container ql-snow" id="some_id" >
    <div class="ql-editor" contenteditable="true" >
        <p>A normal line</p>
        <p class="ql-align-center">a centred line</p>
            <p>A normal line</p><p><br></p><p>A normal line</p>
            <p class="ql-align-center">a centred line</p>
            <p class="ql-align-center">and another one</p>
            <p>A normal line</p>
            <p>formats as</p><p><br></p></div>
        <p>A normal line</p>
        <p class="ql-align-center">a centred line</p>
        <p>A normal line</p>
        <p><br></p>
        <br>
        <p><br></p>
    </div>
</div>
```

## indent

### the text

```
A normal line<br>
-----an indented one<br>
------------indented twice<br>
```

### Formats as Delta document

```json
{
  "ops": [
    { "insert": "A normal line\nan indented one" },
    { "attributes": { "indent": 1 }, "insert": "\n" },
    { "insert": "indented twice" },
    { "attributes": { "indent": 2 }, "insert": "\n" }
  ]
}
```

### with `HTML`

```html
<div class="ql-container ql-snow" id="some_id" >
    <div class="ql-editor" contenteditable="true" >
        <p>A normal line</p>
        <p class="ql-indent-1">an indented one</p>
        <p class="ql-indent-2">indented twice</p>
    </div>
</div>
```

## Ordered list

### The text

```
We list things
  1. first
  2. second
  3. third
```

### Formats as Delta document

```json
{
  "ops": [
    { "insert": "We list things\nfirst " },
    { "attributes": { "list": "ordered" }, "insert": "\n" },
    { "insert": "second " },
    { "attributes": { "list": "ordered" }, "insert": "\n" },
    { "insert": "third" },
    { "attributes": { "list": "ordered" }, "insert": "\n" }
  ]
}
```

### with HTML

```html
<div class="ql-container ql-snow" id="some_id" >
    <div class="ql-editor" contenteditable="true" ><p>We list things</p>
        <ol>
            <li>first </li>
            <li>second </li>
            <li>third</li>
        </ol>
    </div>
</div>
```

## Format and attributes

### The text

```
hello *sweet* world
```

is formatted in red background with the word sweet also bald

### Formats as Delta document

```json
{
  "ops": [
    { "attributes": { "background": "#e60000" }, "insert": "hello " },
    {
      "attributes": { "background": "#e60000", "bold": "true" },
      "insert": "sweet"
    },
    { "attributes": { "background": "#e60000" }, "insert": " world" },
    { "insert": "↵↵" }
  ]
}
```

### with HTML

```html
<div class="ql-container ql-snow" id="some_id" >
    <div class="ql-editor" contenteditable="true" >
        <p>
            <span style="background-color: rgb(230, 0, 0);">hello </span>
            <strong style="background-color: rgb(230, 0, 0);">sweet</strong>
            <span style="background-color: rgb(230, 0, 0);"> world</span>
        </p>
        <p>
            <span style="background-color: rgb(230, 0, 0);">
            <span class="ql-cursor">﻿</span></span>
        </p>
        <p>
            <span style="background-color: rgb(230, 0, 0);">hello </span>
            <strong style="background-color: rgb(230, 0, 0);">sweet</strong>
            <span style="background-color: rgb(230, 0, 0);"> world</span>
        </p>
    </div>
</div>
```
