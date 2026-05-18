# html-extract

Command line tool for processing HTML using CSS selectors.

Reads from stdin, prints to stdout, and allows the user to filter parts of the page using [CSS selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors).

Inspired by [jq](http://stedolan.github.io/jq/).

## Install

### One-line install (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/steven0lisa/html-extract/main/install.sh | bash
```

Install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/steven0lisa/html-extract/main/install.sh | bash -s -- v0.1.0
```

The script auto-detects your OS and architecture, downloads the latest release, and installs to `/usr/local/bin` (or `~/.local/bin` if no write permission).

### Windows

Download the `.exe` from the [releases page](https://github.com/steven0lisa/html-extract/releases/latest) and add it to your PATH.

### Build from source

```bash
git clone https://github.com/steven0lisa/html-extract.git
cd html-extract
cargo build --release
# binary at target/release/html-extract
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
| `--slim` | Output minimal DOM skeleton (ultra-compressed) |
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

### Slim Mode (`--slim`)

Output an ultra-compressed DOM skeleton that only preserves elements with identifying attributes (`id`, `class`, `name`, `type`, `value`, `data-*`). Designed for feeding HTML structure to AI with minimal token cost.

**Tag abbreviations:** `div→d`, `span→s`, `table→t`, `ul→u`, `ol→o`, `li→l`, `section→sec`, `article→art`, `header→hdr`, `footer→ftr`, `main→mn`, `aside→asd`, `button→btn`, `label→lbl`, etc.

```bash
# Full page skeleton
$ curl -s https://example.com | html-extract --slim

# With selector - compress a subtree
$ curl -s https://example.com | html-extract --slim 'div.article-list'

# Custom indent
$ curl -s https://example.com | html-extract --slim -i 4
```

**Input:**
```html
<html>
  <body>
    <div id="main" class="container">
      <h1 class="title">Hello</h1>
      <form class="search-form">
        <input type="text" name="q" data-role="search">
        <button type="submit" class="btn primary">Go</button>
      </form>
    </div>
  </body>
</html>
```

**Output:**
```
d#main.container
  h1.title
  form.search-form
    input[type=text][name=q][data-role=search]
    btn[type=submit].btn.primary
```

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

# Slim mode - ultra-compressed DOM skeleton
$ curl -s https://news.ycombinator.com | html-extract --slim
$ cat index.html | html-extract --slim 'div#content'
```

## License

MIT
