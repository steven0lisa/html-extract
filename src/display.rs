use colored::Colorize;
use ego_tree::NodeId;
use scraper::{Html, Node};
use serde_json::Map;

use crate::Config;

pub trait Displayer {
    fn display(&self, doc: &Html, nodes: &[NodeId], config: &Config);
}

// Parse displayer command from the last selector token
pub fn parse_displayer(cmd: &str) -> Result<DisplayType, String> {
    if cmd == "json{}" {
        return Ok(DisplayType::Json);
    }

    let text_re = regex::Regex::new(r"text\{([\w\+]*)\}").unwrap();
    if let Some(caps) = text_re.captures(cmd) {
        let mods_str = caps[1].to_string();
        let mods: Vec<String> = if mods_str.is_empty() {
            vec![]
        } else {
            mods_str.split('+').map(|s| s.to_string()).collect()
        };
        return Ok(DisplayType::Text { mods });
    }

    let attr_re = regex::Regex::new(r#"attr\{([^\s"'/>=\x00-\x1F]+)\}"#).unwrap();
    if let Some(caps) = attr_re.captures(cmd) {
        return Ok(DisplayType::Attr { attr: caps[1].to_string() });
    }

    Err("Unknown displayer".to_string())
}

#[derive(Debug, Clone)]
pub enum DisplayType {
    Text { mods: Vec<String> },
    Attr { attr: String },
    Json,
}

pub fn create_displayer(cmds: &[String], last_is_displayer: bool) -> Box<dyn Displayer> {
    if !last_is_displayer || cmds.is_empty() {
        return Box::new(TreeDisplayer);
    }

    let last_cmd = cmds.last().unwrap();
    match parse_displayer(last_cmd) {
        Ok(DisplayType::Json) => Box::new(JsonDisplayer),
        Ok(DisplayType::Text { mods }) => Box::new(TextDisplayer { mods }),
        Ok(DisplayType::Attr { attr }) => Box::new(AttrDisplayer { attr }),
        _ => Box::new(TreeDisplayer),
    }
}

// Void elements (self-closing tags)
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area" | "base" | "br" | "col" | "command" | "embed"
            | "hr" | "img" | "input" | "keygen" | "link"
            | "meta" | "param" | "source" | "track" | "wbr"
    )
}

fn escape_html_text(s: &str, escape: bool) -> String {
    if escape {
        html_escape::encode_text(s).to_string()
    } else {
        s.to_string()
    }
}

// Tree Displayer
pub struct TreeDisplayer;

impl Displayer for TreeDisplayer {
    fn display(&self, doc: &Html, nodes: &[NodeId], config: &Config) {
        for &node_id in nodes {
            self.print_node(doc, node_id, 0, config);
        }
    }
}

impl TreeDisplayer {
    fn print_node(&self, doc: &Html, node_id: NodeId, level: i32, config: &Config) {
        let tree_node = doc.tree.get(node_id).unwrap();
        let node = tree_node.value();

        match node {
            Node::Text(text) => {
                let s = escape_html_text(&text.text, config.escape_html);
                if config.raw {
                    print!("{}", s);
                } else {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        self.print_indent(level, config);
                        println!("{}", trimmed);
                    }
                }
            }
            Node::Element(element) => {
                let tag = element.name();
                if !config.raw {
                    self.print_indent(level, config);
                }

                // Handle <pre> with preformatted option
                if tag == "pre" && !config.color && config.pre {
                    self.print_pre(doc, node_id, config);
                    if !config.raw {
                        println!();
                    }
                    return;
                }

                if config.color {
                    print!("{}", "<".cyan());
                    print!("{}", tag.cyan());
                } else {
                    print!("<{}", tag);
                }

                for attr in element.attrs() {
                    let val = escape_html_text(attr.1.as_ref(), config.escape_html);
                    if config.color {
                        print!(" ");
                        print!("{}", attr.0.to_string().magenta());
                        print!("{}", "=".blue());
                        print!("{}", format!("\"{}\"", val).blue());
                    } else {
                        print!(" {}=\"{}\"", attr.0, val);
                    }
                }

                if config.color {
                    print!("{}", ">".cyan());
                } else {
                    print!(">");
                }

                if !config.raw {
                    println!();
                }

                if !is_void_element(tag) {
                    self.print_children(doc, node_id, level + 1, config);
                    if !config.raw {
                        self.print_indent(level, config);
                    }
                    if config.color {
                        print!("{}", "</".cyan());
                        print!("{}", tag.cyan());
                        print!("{}", ">".cyan());
                    } else {
                        print!("</{}>", tag);
                    }
                    if !config.raw {
                        println!();
                    }
                }
            }
            Node::Comment(comment) => {
                if !config.raw {
                    self.print_indent(level, config);
                }
                let data = escape_html_text(&comment.comment, config.escape_html);
                if config.color {
                    println!("{}", format!("<!--{}-->", data).yellow());
                } else {
                    println!("<!--{}-->", data);
                }
                self.print_children(doc, node_id, level, config);
            }
            Node::Document | Node::Doctype(_) => {
                self.print_children(doc, node_id, level, config);
            }
            _ => {}
        }
    }

    fn print_children(&self, doc: &Html, node_id: NodeId, level: i32, config: &Config) {
        if config.max_print_level > -1 && level >= config.max_print_level {
            self.print_indent(level, config);
            println!("...");
            return;
        }
        let tree_node = doc.tree.get(node_id).unwrap();
        for child in tree_node.children() {
            self.print_node(doc, child.id(), level, config);
        }
    }

    fn print_indent(&self, level: i32, config: &Config) {
        for _ in 0..level {
            print!("{}", config.indent_string);
        }
    }

    fn print_pre(&self, doc: &Html, node_id: NodeId, config: &Config) {
        let tree_node = doc.tree.get(node_id).unwrap();
        let node = tree_node.value();

        match node {
            Node::Text(text) => {
                let s = escape_html_text(&text.text, config.escape_html);
                print!("{}", s);
                for child in tree_node.children() {
                    self.print_pre(doc, child.id(), config);
                }
            }
            Node::Element(element) => {
                let tag = element.name();
                print!("<{}", tag);
                for attr in element.attrs() {
                    let val = escape_html_text(attr.1.as_ref(), config.escape_html);
                    print!(" {}=\"{}\"", attr.0, val);
                }
                print!(">");
                if !is_void_element(tag) {
                    for child in tree_node.children() {
                        self.print_pre(doc, child.id(), config);
                    }
                    print!("</{}>", tag);
                }
            }
            Node::Comment(comment) => {
                let data = escape_html_text(&comment.comment, config.escape_html);
                print!("<!--{}-->", data);
                if !config.raw {
                    println!();
                }
                for child in tree_node.children() {
                    self.print_pre(doc, child.id(), config);
                }
            }
            Node::Document | Node::Doctype(_) => {
                for child in tree_node.children() {
                    self.print_pre(doc, child.id(), config);
                }
            }
            _ => {}
        }
    }
}

// Text Displayer
pub struct TextDisplayer {
    pub mods: Vec<String>,
}

impl Displayer for TextDisplayer {
    fn display(&self, doc: &Html, nodes: &[NodeId], config: &Config) {
        for &node_id in nodes {
            self.display_node(doc, node_id, config);
        }
    }
}

impl TextDisplayer {
    fn display_node(&self, doc: &Html, node_id: NodeId, config: &Config) {
        let tree_node = doc.tree.get(node_id).unwrap();
        let node = tree_node.value();

        if let Node::Text(text) = node {
            let mut data = escape_html_text(&text.text, config.escape_html);
            for mod_name in &self.mods {
                match mod_name.as_str() {
                    "" => {}
                    "trim" => data = data.trim().to_string(),
                    "lower" => data = data.to_lowercase(),
                    "upper" => data = data.to_uppercase(),
                    _ => eprintln!("Text modifier '{}' not recognized, ignoring", mod_name),
                }
            }
            println!("{}", data);
        }

        let children: Vec<NodeId> = tree_node.children().map(|c| c.id()).collect();
        for child_id in children {
            self.display_node(doc, child_id, config);
        }
    }
}

// Attr Displayer
pub struct AttrDisplayer {
    pub attr: String,
}

impl Displayer for AttrDisplayer {
    fn display(&self, doc: &Html, nodes: &[NodeId], config: &Config) {
        for &node_id in nodes {
            let tree_node = doc.tree.get(node_id).unwrap();
            if let Some(element) = tree_node.value().as_element() {
                for attr in element.attrs() {
                    if attr.0.as_ref() == self.attr {
                        let val = escape_html_text(attr.1.as_ref(), config.escape_html);
                        print!("{}", val);
                        if !config.raw {
                            println!();
                        }
                    }
                }
            }
        }
    }
}

// JSON Displayer
pub struct JsonDisplayer;

impl Displayer for JsonDisplayer {
    fn display(&self, doc: &Html, nodes: &[NodeId], config: &Config) {
        let json_nodes: Vec<Map<String, serde_json::Value>> =
            nodes.iter().map(|&node_id| self.jsonify(doc, node_id, config)).collect();

        let json_str = serde_json::to_string_pretty(&json_nodes).unwrap();
        // Replace the default 2-space indent with the configured indent
        let json_str = if config.indent_string != "  " {
            json_str.replace("  ", &config.indent_string)
        } else {
            json_str
        };
        print!("{}", json_str);
        if !config.raw {
            println!();
        }
    }
}

impl JsonDisplayer {
    fn jsonify(&self, doc: &Html, node_id: NodeId, config: &Config) -> Map<String, serde_json::Value> {
        let mut vals = Map::new();
        let tree_node = doc.tree.get(node_id).unwrap();

        if let Some(element) = tree_node.value().as_element() {
            // Add tag
            vals.insert("tag".to_string(), serde_json::Value::String(element.name().to_string()));

            // Add attributes
            if element.attrs().count() > 0 {
                for attr in element.attrs() {
                    let val = if config.escape_html {
                        html_escape::encode_text::<str>(&attr.1).to_string()
                    } else {
                        attr.1.to_string()
                    };
                    vals.insert(attr.0.to_string(), serde_json::Value::String(val));
                }
            }

            // Add children and text
            let mut children: Vec<serde_json::Value> = Vec::new();
            let mut text_parts: Vec<String> = Vec::new();
            let mut comment_parts: Vec<String> = Vec::new();

            for child in tree_node.children() {
                match child.value() {
                    Node::Element(_) => {
                        children.push(serde_json::Value::Object(self.jsonify(doc, child.id(), config)));
                    }
                    Node::Text(text_node) => {
                        let text = text_node.text.trim().to_string();
                        if !text.is_empty() {
                            let escaped = if config.escape_html && element.name() != "script" {
                                html_escape::encode_text(&text).to_string()
                            } else {
                                text
                            };
                            text_parts.push(escaped);
                        }
                    }
                    Node::Comment(comment) => {
                        let comment_text = comment.comment.trim().to_string();
                        let escaped = if config.escape_html {
                            html_escape::encode_text(&comment_text).to_string()
                        } else {
                            comment_text
                        };
                        comment_parts.push(escaped);
                    }
                    _ => {}
                }
            }

            if !text_parts.is_empty() {
                vals.insert("text".to_string(), serde_json::Value::String(text_parts.join(" ")));
            }
            if !comment_parts.is_empty() {
                vals.insert("comment".to_string(), serde_json::Value::String(comment_parts.join(" ")));
            }
            if !children.is_empty() {
                vals.insert("children".to_string(), serde_json::Value::Array(children));
            }
        }

        vals
    }
}

// Slim Displayer - minimal DOM skeleton output
pub struct SlimDisplayer;

fn has_identifier_attrs(element: &scraper::node::Element) -> bool {
    for attr in element.attrs() {
        let key: &str = attr.0.as_ref();
        if key == "id" || key == "class" || key == "name" || key == "type" || key == "value" || key.starts_with("data-") {
            return true;
        }
    }
    false
}

impl Displayer for SlimDisplayer {
    fn display(&self, doc: &Html, nodes: &[NodeId], config: &Config) {
        for &node_id in nodes {
            self.print_slim_node(doc, node_id, 0, config);
        }
    }
}

impl SlimDisplayer {
    fn print_slim_node(&self, doc: &Html, node_id: NodeId, effective_level: i32, config: &Config) {
        let tree_node = doc.tree.get(node_id).unwrap();
        let node = tree_node.value();

        if let Node::Element(element) = node {
            if has_identifier_attrs(element) {
                let mut line = String::new();

                // Indent
                for _ in 0..effective_level {
                    line.push_str(&config.indent_string);
                }

                // Collect attrs: id, class, then name/type/value/data-*
                let mut id_val: Option<&str> = None;
                let mut class_val: Option<&str> = None;
                let mut extra_attrs: Vec<(&str, &str)> = Vec::new();

                for attr in element.attrs() {
                    let key = attr.0.as_ref();
                    let val = attr.1.as_ref();
                    match key {
                        "id" => id_val = Some(val),
                        "class" => class_val = Some(val),
                        "name" | "type" | "value" => {
                            extra_attrs.push((key, val));
                        }
                        k if k.starts_with("data-") => {
                            extra_attrs.push((k, val));
                        }
                        _ => {}
                    }
                }

                // id → #id
                if let Some(id) = id_val {
                    line.push('#');
                    line.push_str(id);
                }

                // class → .class1.class2
                if let Some(classes) = class_val {
                    for cls in classes.split_whitespace() {
                        line.push('.');
                        line.push_str(cls);
                    }
                }

                // Extra attrs: name, type, value, data-*
                for (key, val) in &extra_attrs {
                    line.push('[');
                    line.push_str(key);
                    line.push('=');
                    line.push_str(val);
                    line.push(']');
                }

                println!("{}", line);
                self.print_slim_children(doc, node_id, effective_level + 1, config);
            } else {
                // No identifier attrs — skip this element but process children at same level
                self.print_slim_children(doc, node_id, effective_level, config);
            }
        } else {
            // Document, Doctype, etc — just process children
            self.print_slim_children(doc, node_id, effective_level, config);
        }
    }

    fn print_slim_children(&self, doc: &Html, node_id: NodeId, level: i32, config: &Config) {
        let tree_node = doc.tree.get(node_id).unwrap();
        for child in tree_node.children() {
            self.print_slim_node(doc, child.id(), level, config);
        }
    }
}
