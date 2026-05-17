# html-extract (pup-rs)

Command line tool for processing HTML using CSS selectors. A Rust port of [pup](https://github.com/ericchiang/pup).

Reads from stdin, prints to stdout, and allows the user to filter parts of the page using [CSS selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors).

Inspired by [jq](http://stedolan.github.io/jq/).

## Install

Download pre-built binaries from the [releases page](https://github.com/steven0lisa/html-extract/releases/latest).

Or build from source:

```bash
cargo build --release
```

## Quick start

```bash
$ curl -s https://news.ycombinator.com/ | html-extract 'table table tr:nth-last-of-type(n+2) td.title a'
```

## Usage

```bash
$ cat index.html | html-extract [flags] [selectors] [display function]
```

### Flags

| Flag | Description |
|------|-------------|
| `-c, --color` | Print result with color |
| `-f, --file <file>` | File to read from |
| `-h, --help` | Display help |
| `-i, --indent <level>` | Number of spaces or character for indent |
| `-n, --number` | Print number of elements selected |
| `-l, --limit <level>` | Restrict number of levels printed |
| `-p, --plain` | Don't escape HTML |
| `-r, --raw` | Raw output (no formatting) |
| `-s, --strict` | Ignore non-standard HTML tags |
| `--pre` | Preserve preformatted text |
| `--charset <charset>` | Specify the charset |
| `--version` | Display version |

### Implemented Selectors

```bash
html-extract '.class'
html-extract '#id'
html-extract 'element'
html-extract 'selector + selector'   # next sibling
html-extract 'selector > selector'   # direct child
html-extract 'selector, selector'    # multiple selectors
html-extract '[attribute]'
html-extract '[attribute="value"]'
html-extract '[attribute*="value"]'   # contains
html-extract '[attribute~="value"]'   # word match
html-extract '[attribute^="value"]'   # starts with
html-extract '[attribute$="value"]'   # ends with
html-extract ':empty'
html-extract ':first-child'
html-extract ':first-of-type'
html-extract ':last-child'
html-extract ':last-of-type'
html-extract ':only-child'
html-extract ':only-of-type'
html-extract ':contains("text")'
html-extract ':matches("pattern")'
html-extract ':nth-child(n)'
html-extract ':nth-of-type(n)'
html-extract ':nth-last-child(n)'
html-extract ':nth-last-of-type(n)'
html-extract ':not(selector)'
html-extract ':parent-of(selector)'
```

### Display Functions

| Function | Description |
|----------|-------------|
| `text{}` | Print all text content |
| `text{trim}` | Trim whitespace |
| `text{upper}` | Convert to uppercase |
| `text{lower}` | Convert to lowercase |
| `text{trim+upper}` | Combine modifiers |
| `attr{name}` | Print attribute value |
| `json{}` | Output as JSON |

## Examples

```bash
# Filter by tag
$ cat index.html | html-extract 'title'

# Filter by id
$ cat index.html | html-extract '#footer'

# Filter by class
$ cat index.html | html-extract '.summary'

# Get text content
$ cat index.html | html-extract 'h1 text{}'

# Get attribute value
$ cat index.html | html-extract 'a attr{href}'

# JSON output
$ cat index.html | html-extract 'div#content json{}'

# Count elements
$ cat index.html | html-extract -n 'a'

# Comma-separated selectors
$ cat index.html | html-extract 'title, h1'

# Child combinator
$ cat index.html | html-extract 'div > p'

# Next sibling
$ cat index.html | html-extract 'h1 + p'

# Pseudo classes
$ cat index.html | html-extract 'li:first-child'
$ cat index.html | html-extract 'li:nth-child(2)'
$ cat index.html | html-extract ':contains("Rob")'
```

## License

MIT
