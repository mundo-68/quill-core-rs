# A better table format

Defines a table format which allows the table to be editted, and have multi line text.
Copy from forum: https://github.com/soccerloway/quill-better-table/issues/45

# Format structure

```javascript
interface TableCellLineAttributes {
  rowspan: string;
  colspan: string;
  row: string;
  cell: string;
}

interface TableCellLine {
  attributes: {
    'table-cell-line': TableCellLineAttributes;
    row: string;
    rowspan: string;
    colspan: string;
  },
  insert: '\n'
}
```

# Example Delta document

See image in the same folder as this document ...

![Resulting table from the below JSON](table_example.png?raw=true&sanitize=true "Resulting table from the below JSON")

```json
{
  "ops": [
    {
      "insert": "\n"
    },
    {
      "attributes": { "table-col": { "width": "100" } },
      "insert": "\n\n\n"
    },
    {
      "insert": "cell-1-line-1"
    },
    {
      "attributes": {
        "table-cell-line": {
          "rowspan": "1",
          "colspan": "1",
          "row": "row-zb2s",
          "cell": "cell-bo8d"
        },
        "row": "row-zb2s",
        "rowspan": "1",
        "colspan": "1"
      },
      "insert": "\n"
    },
    {
      "insert": "cell-1-line-2"
    },
    {
      "attributes": {
        "table-cell-line": {
          "rowspan": "1",
          "colspan": "1",
          "row": "row-zb2s",
          "cell": "cell-bo8d"
        },
        "row": "row-zb2s",
        "rowspan": "1",
        "colspan": "1"
      },
      "insert": "\n"
    },
    {
      "insert": "cell-2-line-1"
    },
    {
      "attributes": {
        "table-cell-line": {
          "rowspan": "1",
          "colspan": "1",
          "row": "row-zb2s",
          "cell": "cell-qxi3"
        },
        "row": "row-zb2s",
        "rowspan": "1",
        "colspan": "1"
      },
      "insert": "\n"
    },
    {
      "insert": "cell-3-line-1"
    },
    {
      "attributes": {
        "table-cell-line": {
          "rowspan": "1",
          "colspan": "1",
          "row": "row-zb2s",
          "cell": "cell-8xwx"
        },
        "row": "row-zb2s",
        "rowspan": "1",
        "colspan": "1"
      },
      "insert": "\n"
    },
    {
      "insert": "cell-4-line-1"
    },
    {
      "attributes": {
        "table-cell-line": {
          "rowspan": "1",
          "colspan": "1",
          "row": "row-62ms",
          "cell": "cell-x1jq"
        },
        "row": "row-62ms",
        "rowspan": "1",
        "colspan": "1"
      },
      "insert": "\n"
    },
    {
      "insert": "cell 5,6,8,9 merged"
    },
    {
      "attributes": {
        "table-cell-line": {
          "rowspan": "2",
          "colspan": "2",
          "row": "row-62ms",
          "cell": "cell-4cgh"
        },
        "row": "row-62ms",
        "rowspan": "2",
        "colspan": "2"
      },
      "insert": "\n"
    },
    {
      "insert": "cell-7-line-1"
    },
    {
      "attributes": {
        "table-cell-line": {
          "rowspan": "1",
          "colspan": "1",
          "row": "row-51hq",
          "cell": "cell-pnz2"
        },
        "row": "row-51hq",
        "rowspan": "1",
        "colspan": "1"
      },
      "insert": "\n"
    },
    {
      "insert": "\n"
    }
  ]
}
```
