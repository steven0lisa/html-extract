use regex::Regex;
use scraper::Html;
use ego_tree::NodeId;

pub type SelectorFunc = Box<dyn Fn(&Html, &[NodeId]) -> Vec<NodeId>>;

/// Parse HTML string into a document
pub fn parse_html(html: &str) -> Result<Html, String> {
    Ok(Html::parse_document(html))
}

pub enum SelectorFuncType {
    Descendant,
    Child,
    NextSibling,
}

// ContextualSelector that works with the document tree
pub struct ContextualSelector {
    tag: Option<String>,
    attrs: Vec<(String, Vec<Regex>)>,
    pseudo: Option<Box<dyn Fn(&Html, NodeId) -> bool>>,
}

impl ContextualSelector {
    pub fn matches(&self, doc: &Html, node_id: NodeId) -> bool {
        let tree_node = doc.tree.get(node_id).unwrap();
        let node = tree_node.value();

        let element = match node.as_element() {
            Some(e) => e,
            None => return false,
        };

        // Check tag
        if let Some(ref tag) = self.tag {
            if element.name() != tag.as_str() {
                return false;
            }
        }

        // Check attributes
        for (attr_key, matchers) in &self.attrs {
            let mut matched = false;
            for attr in element.attrs() {
                if *attr_key == attr.0.as_ref() {
                    for matcher in matchers {
                        if !matcher.is_match(attr.1.as_ref()) {
                            return false;
                        }
                    }
                    matched = true;
                    break;
                }
            }
            if !matched {
                return false;
            }
        }

        // Check pseudo class
        if let Some(ref pseudo) = self.pseudo {
            if !pseudo(doc, node_id) {
                return false;
            }
        }

        true
    }
}

/// Select: find all descendants matching the selector
fn select_with_context(s: ContextualSelector) -> SelectorFunc {
    Box::new(move |doc: &Html, nodes: &[NodeId]| -> Vec<NodeId> {
        let mut selected: Vec<NodeId> = Vec::new();
        for &node_id in nodes {
            select_children_contextual(doc, node_id, &s, &mut selected);
        }
        selected
    })
}

fn select_children_contextual(doc: &Html, node_id: NodeId, s: &ContextualSelector, selected: &mut Vec<NodeId>) {
    let node = doc.tree.get(node_id).unwrap();
    for child in node.children() {
        let child_id = child.id();
        if s.matches(doc, child_id) {
            selected.push(child_id);
        } else {
            select_children_contextual(doc, child_id, s, selected);
        }
    }
}

/// SelectFromChildren: '>' - only direct children
fn select_from_children_with_context(s: ContextualSelector) -> SelectorFunc {
    Box::new(move |doc: &Html, nodes: &[NodeId]| -> Vec<NodeId> {
        let mut selected: Vec<NodeId> = Vec::new();
        for &node_id in nodes {
            let node = doc.tree.get(node_id).unwrap();
            for child in node.children() {
                let child_id = child.id();
                if s.matches(doc, child_id) {
                    selected.push(child_id);
                }
            }
        }
        selected
    })
}

/// SelectNextSibling: '+' - next sibling element
fn select_next_sibling_with_context(s: ContextualSelector) -> SelectorFunc {
    Box::new(move |doc: &Html, nodes: &[NodeId]| -> Vec<NodeId> {
        let mut selected: Vec<NodeId> = Vec::new();
        for &node_id in nodes {
            let node = doc.tree.get(node_id).unwrap();
            for sibling in node.next_siblings() {
                let sibling_id = sibling.id();
                let sibling_node = sibling.value();
                if sibling_node.as_element().is_some() {
                    if s.matches(doc, sibling_id) {
                        selected.push(sibling_id);
                    }
                    break;
                }
            }
        }
        selected
    })
}

/// Main entry point for building selector functions
pub fn build_selector_func(cmd: &str, func_type: SelectorFuncType, _strict: bool) -> Result<SelectorFunc, String> {
    let sel = parse_contextual_selector(cmd)?;
    Ok(match func_type {
        SelectorFuncType::Descendant => select_with_context(sel),
        SelectorFuncType::Child => select_from_children_with_context(sel),
        SelectorFuncType::NextSibling => select_next_sibling_with_context(sel),
    })
}

fn parse_contextual_selector(cmd: &str) -> Result<ContextualSelector, String> {
    let chars: Vec<char> = cmd.chars().collect();
    let mut pos = 0;

    let mut tag: Option<String> = None;
    let mut attrs: Vec<(String, Vec<Regex>)> = Vec::new();
    let mut pseudo: Option<Box<dyn Fn(&Html, NodeId) -> bool>> = None;

    // Parse tag
    let mut tag_buf = String::new();
    while pos < chars.len() {
        match chars[pos] {
            '.' | '#' | '[' | ':' => break,
            _ => {
                tag_buf.push(chars[pos]);
                pos += 1;
            }
        }
    }
    if !tag_buf.is_empty() {
        tag = Some(tag_buf);
    }

    // Parse remaining parts
    while pos < chars.len() {
        match chars[pos] {
            '.' => {
                pos += 1;
                let class_name = parse_identifier(&chars, &mut pos)?;
                let re = Regex::new(&format!(r"(^|\s){}(\s|$)", regex::escape(&class_name))).unwrap();
                attrs.push(("class".to_string(), vec![re]));
            }
            '#' => {
                pos += 1;
                let id_name = parse_identifier(&chars, &mut pos)?;
                let re = Regex::new(&format!(r"^{}$", regex::escape(&id_name))).unwrap();
                attrs.push(("id".to_string(), vec![re]));
            }
            '[' => {
                pos += 1;
                let (key, re) = parse_attr_matcher(&chars, &mut pos)?;
                attrs.push((key, vec![re]));
            }
            ':' => {
                pos += 1;
                pseudo = Some(parse_pseudo_class(&chars, &mut pos)?);
            }
            _ => {
                return Err(format!("Unexpected character '{}' in selector", chars[pos]));
            }
        }
    }

    Ok(ContextualSelector { tag, attrs, pseudo })
}

fn parse_identifier(chars: &[char], pos: &mut usize) -> Result<String, String> {
    let mut buf = String::new();
    while *pos < chars.len() {
        match chars[*pos] {
            '.' | '#' | '[' | ':' | ' ' => break,
            _ => {
                buf.push(chars[*pos]);
                *pos += 1;
            }
        }
    }
    Ok(buf)
}

fn parse_attr_matcher(chars: &[char], pos: &mut usize) -> Result<(String, Regex), String> {
    let mut key = String::new();
    let mut match_type: u8 = b'=';

    // Parse key and match type
    while *pos < chars.len() {
        match chars[*pos] {
            ']' => {
                *pos += 1;
                let re = Regex::new(r"^.*$").unwrap();
                return Ok((key, re));
            }
            '$' | '^' | '~' | '*' => {
                match_type = chars[*pos] as u8;
                *pos += 1;
                if *pos >= chars.len() || chars[*pos] != '=' {
                    return Err(format!("'{}' must be followed by a '='", match_type as char));
                }
                *pos += 1;
                break;
            }
            '=' => {
                *pos += 1;
                break;
            }
            _ => {
                key.push(chars[*pos]);
                *pos += 1;
            }
        }
    }

    // Parse value
    let mut val = String::new();
    if *pos < chars.len() && chars[*pos] == '"' {
        *pos += 1; // skip opening quote
        while *pos < chars.len() {
            match chars[*pos] {
                '\\' => {
                    *pos += 1;
                    if *pos >= chars.len() {
                        return Err("Unmatched open brace '['".to_string());
                    }
                    val.push(chars[*pos]);
                    *pos += 1;
                }
                '"' => {
                    *pos += 1;
                    break;
                }
                _ => {
                    val.push(chars[*pos]);
                    *pos += 1;
                }
            }
        }
        if *pos >= chars.len() || chars[*pos] != ']' {
            return Err("Quote must end at ']'".to_string());
        }
        *pos += 1;
    } else {
        while *pos < chars.len() {
            if chars[*pos] == ']' {
                *pos += 1;
                break;
            }
            val.push(chars[*pos]);
            *pos += 1;
        }
    }

    let regexp_str = match match_type as char {
        '=' => format!("^{}$", regex::escape(&val)),
        '*' => regex::escape(&val),
        '$' => format!("{}$", regex::escape(&val)),
        '^' => format!("^{}", regex::escape(&val)),
        '~' => format!(r"(^|\s){}(\s|$)", regex::escape(&val)),
        _ => return Err("Invalid match type".to_string()),
    };

    let re = Regex::new(&regexp_str).map_err(|e| e.to_string())?;
    Ok((key, re))
}

fn parse_pseudo_class(chars: &[char], pos: &mut usize) -> Result<Box<dyn Fn(&Html, NodeId) -> bool>, String> {
    let rest: String = chars[*pos..].iter().collect();
    *pos = chars.len();

    match rest.as_str() {
        "empty" => Ok(Box::new(|doc: &Html, node_id: NodeId| -> bool {
            let node = doc.tree.get(node_id).unwrap();
            node.children().all(|c| !c.value().is_element() && {
                if let Some(text) = c.value().as_text() {
                    text.text.trim().is_empty()
                } else {
                    true
                }
            })
        })),
        "first-child" => Ok(Box::new(|doc: &Html, node_id: NodeId| -> bool {
            let node = doc.tree.get(node_id).unwrap();
            for prev in node.prev_siblings() {
                if prev.value().is_element() {
                    return false;
                }
            }
            true
        })),
        "last-child" => Ok(Box::new(|doc: &Html, node_id: NodeId| -> bool {
            let node = doc.tree.get(node_id).unwrap();
            for next_node in node.next_siblings() {
                if next_node.value().is_element() {
                    return false;
                }
            }
            true
        })),
        "only-child" => Ok(Box::new(|doc: &Html, node_id: NodeId| -> bool {
            let node = doc.tree.get(node_id).unwrap();
            let has_prev = node.prev_siblings().any(|s| s.value().is_element());
            let has_next = node.next_siblings().any(|s| s.value().is_element());
            !has_prev && !has_next
        })),
        "first-of-type" => Ok(Box::new(|doc: &Html, node_id: NodeId| -> bool {
            let node = doc.tree.get(node_id).unwrap();
            let tag = match node.value().as_element() {
                Some(e) => e.name(),
                None => return false,
            };
            for prev in node.prev_siblings() {
                if let Some(e) = prev.value().as_element() {
                    if e.name() == tag {
                        return false;
                    }
                }
            }
            true
        })),
        "last-of-type" => Ok(Box::new(|doc: &Html, node_id: NodeId| -> bool {
            let node = doc.tree.get(node_id).unwrap();
            let tag = match node.value().as_element() {
                Some(e) => e.name(),
                None => return false,
            };
            for next_node in node.next_siblings() {
                if let Some(e) = next_node.value().as_element() {
                    if e.name() == tag {
                        return false;
                    }
                }
            }
            true
        })),
        "only-of-type" => Ok(Box::new(|doc: &Html, node_id: NodeId| -> bool {
            let node = doc.tree.get(node_id).unwrap();
            let tag = match node.value().as_element() {
                Some(e) => e.name(),
                None => return false,
            };
            let has_prev = node.prev_siblings().any(|s| {
                s.value().as_element().map(|e| e.name() == tag).unwrap_or(false)
            });
            let has_next = node.next_siblings().any(|s| {
                s.value().as_element().map(|e| e.name() == tag).unwrap_or(false)
            });
            !has_prev && !has_next
        })),
        _ => {
            if rest.starts_with("contains(") {
                parse_contains_pseudo(&rest["contains(".len()..])
            } else if rest.starts_with("matches(") {
                parse_matches_pseudo(&rest["matches(".len()..])
            } else if rest.starts_with("nth-child(")
                || rest.starts_with("nth-last-child(")
                || rest.starts_with("nth-of-type(")
                || rest.starts_with("nth-last-of-type(")
            {
                parse_nth_pseudo(&rest)
            } else if rest.starts_with("not(") {
                parse_not_pseudo(&rest["not(".len()..])
            } else if rest.starts_with("parent-of(") {
                parse_parent_of_pseudo(&rest["parent-of(".len()..])
            } else {
                Err(format!("{} not a valid pseudo class", rest))
            }
        }
    }
}

fn parse_contains_pseudo(cmd: &str) -> Result<Box<dyn Fn(&Html, NodeId) -> bool>, String> {
    let chars: Vec<char> = cmd.chars().collect();
    let mut pos = 0;

    if chars.get(pos) != Some(&'"') {
        return Err("Malformed 'contains(\"\")' selector".to_string());
    }
    pos += 1;

    let mut text = String::new();
    while pos < chars.len() {
        match chars[pos] {
            '\\' => {
                pos += 1;
                if pos >= chars.len() {
                    return Err("Malformed 'contains(\"\")' selector".to_string());
                }
                text.push(chars[pos]);
                pos += 1;
            }
            '"' => {
                pos += 1;
                if chars.get(pos) != Some(&')') {
                    return Err("Malformed 'contains(\"\")' selector".to_string());
                }
                pos += 1;
                if pos < chars.len() {
                    return Err("'contains(\"\")' must end selector".to_string());
                }
                let text_to_contain = text;
                return Ok(Box::new(move |doc: &Html, node_id: NodeId| -> bool {
                    let node = doc.tree.get(node_id).unwrap();
                    for child in node.children() {
                        if let Some(text_node) = child.value().as_text() {
                            if text_node.text.contains(&text_to_contain) {
                                return true;
                            }
                        }
                    }
                    false
                }));
            }
            _ => {
                text.push(chars[pos]);
                pos += 1;
            }
        }
    }
    Err("Malformed 'contains(\"\")' selector".to_string())
}

fn parse_matches_pseudo(cmd: &str) -> Result<Box<dyn Fn(&Html, NodeId) -> bool>, String> {
    let chars: Vec<char> = cmd.chars().collect();
    let mut pos = 0;

    if chars.get(pos) != Some(&'"') {
        return Err("Malformed 'matches(\"\")' selector".to_string());
    }
    pos += 1;

    let mut pattern = String::new();
    while pos < chars.len() {
        match chars[pos] {
            '\\' => {
                pos += 1;
                if pos >= chars.len() {
                    return Err("Malformed 'matches(\"\")' selector".to_string());
                }
                pattern.push(chars[pos]);
                pos += 1;
            }
            '"' => {
                pos += 1;
                if chars.get(pos) != Some(&')') {
                    return Err("Malformed 'matches(\"\")' selector".to_string());
                }
                pos += 1;
                if pos < chars.len() {
                    return Err("'matches(\"\")' must end selector".to_string());
                }
                let re = Regex::new(&pattern).map_err(|e| e.to_string())?;
                return Ok(Box::new(move |doc: &Html, node_id: NodeId| -> bool {
                    let node = doc.tree.get(node_id).unwrap();
                    for child in node.children() {
                        if let Some(text_node) = child.value().as_text() {
                            if re.is_match(&text_node.text) {
                                return true;
                            }
                        }
                    }
                    false
                }));
            }
            _ => {
                pattern.push(chars[pos]);
                pos += 1;
            }
        }
    }
    Err("Malformed 'matches(\"\")' selector".to_string())
}

fn parse_nth_pseudo(cmd: &str) -> Result<Box<dyn Fn(&Html, NodeId) -> bool>, String> {
    let open_paren = cmd.find('(').ok_or("Fatal error, no '(' found")?;
    let pseudo_name = &cmd[..open_paren];
    let after_paren = &cmd[open_paren + 1..];
    let close_paren = after_paren.find(')').ok_or("Unmatched '(' for pseudo class")?;

    if close_paren != after_paren.len() - 1 {
        return Err(format!("{}(n) must end selector", pseudo_name));
    }

    let number = &after_paren[..close_paren];

    let count_fn: Box<dyn Fn(&Html, NodeId) -> i32> = match pseudo_name {
        "nth-child" => Box::new(|doc: &Html, node_id: NodeId| -> i32 {
            let node = doc.tree.get(node_id).unwrap();
            let mut nth = 1i32;
            for prev in node.prev_siblings() {
                if prev.value().is_element() {
                    nth += 1;
                }
            }
            nth
        }),
        "nth-of-type" => Box::new(|doc: &Html, node_id: NodeId| -> i32 {
            let node = doc.tree.get(node_id).unwrap();
            let tag = node.value().as_element().map(|e| e.name()).unwrap_or("");
            let mut nth = 1i32;
            for prev in node.prev_siblings() {
                if let Some(e) = prev.value().as_element() {
                    if e.name() == tag {
                        nth += 1;
                    }
                }
            }
            nth
        }),
        "nth-last-child" => Box::new(|doc: &Html, node_id: NodeId| -> i32 {
            let node = doc.tree.get(node_id).unwrap();
            let mut nth = 1i32;
            for next_node in node.next_siblings() {
                if next_node.value().is_element() {
                    nth += 1;
                }
            }
            nth
        }),
        "nth-last-of-type" => Box::new(|doc: &Html, node_id: NodeId| -> i32 {
            let node = doc.tree.get(node_id).unwrap();
            let tag = node.value().as_element().map(|e| e.name()).unwrap_or("");
            let mut nth = 1i32;
            for next_node in node.next_siblings() {
                if let Some(e) = next_node.value().as_element() {
                    if e.name() == tag {
                        nth += 1;
                    }
                }
            }
            nth
        }),
        _ => return Err(format!("Unrecognized pseudo '{}'", pseudo_name)),
    };

    match number {
        "odd" => {
            Ok(Box::new(move |doc: &Html, node_id: NodeId| -> bool {
                let node = doc.tree.get(node_id).unwrap();
                node.value().is_element() && count_fn(doc, node_id) % 2 == 1
            }))
        }
        "even" => {
            Ok(Box::new(move |doc: &Html, node_id: NodeId| -> bool {
                let node = doc.tree.get(node_id).unwrap();
                node.value().is_element() && count_fn(doc, node_id) % 2 == 0
            }))
        }
        _ => {
            // Check '3n+4' pattern
            let cycle_re = Regex::new(r"(\d+)n\s?\+\s?(\d)").unwrap();
            if let Some(caps) = cycle_re.captures(number) {
                let cycle: i32 = caps[1].parse().unwrap();
                let offset: i32 = caps[2].parse().unwrap();
                return Ok(Box::new(move |doc: &Html, node_id: NodeId| -> bool {
                    let node = doc.tree.get(node_id).unwrap();
                    node.value().is_element() && count_fn(doc, node_id) % cycle == offset
                }));
            }

            // Check 'n+2' pattern
            let n_plus_re = Regex::new(r"n\s?\+\s?(\d)").unwrap();
            if let Some(caps) = n_plus_re.captures(number) {
                let offset: i32 = caps[1].parse().unwrap();
                return Ok(Box::new(move |doc: &Html, node_id: NodeId| -> bool {
                    let node = doc.tree.get(node_id).unwrap();
                    node.value().is_element() && count_fn(doc, node_id) >= offset
                }));
            }

            // Plain number
            match number.parse::<i32>() {
                Ok(nth) if nth > 0 => {
                    Ok(Box::new(move |doc: &Html, node_id: NodeId| -> bool {
                        let node = doc.tree.get(node_id).unwrap();
                        node.value().is_element() && count_fn(doc, node_id) == nth
                    }))
                }
                Ok(_) => Err(format!("Argument to '{}' must be greater than 0", pseudo_name)),
                Err(_) => Err(format!("Invalid argument '{}' for '{}'", number, pseudo_name)),
            }
        }
    }
}

fn parse_not_pseudo(cmd: &str) -> Result<Box<dyn Fn(&Html, NodeId) -> bool>, String> {
    if cmd.len() < 2 {
        return Err("malformed ':not' selector".to_string());
    }
    let end_char = cmd.chars().last().unwrap();
    let inner = &cmd[..cmd.len() - 1];
    if end_char != ')' {
        return Err("unmatched '('".to_string());
    }
    let sel = parse_contextual_selector(inner)?;
    Ok(Box::new(move |doc: &Html, node_id: NodeId| -> bool {
        !sel.matches(doc, node_id)
    }))
}

fn parse_parent_of_pseudo(cmd: &str) -> Result<Box<dyn Fn(&Html, NodeId) -> bool>, String> {
    if cmd.len() < 2 {
        return Err("malformed ':parent-of' selector".to_string());
    }
    let end_char = cmd.chars().last().unwrap();
    let inner = &cmd[..cmd.len() - 1];
    if end_char != ')' {
        return Err("unmatched '('".to_string());
    }
    let sel = parse_contextual_selector(inner)?;
    Ok(Box::new(move |doc: &Html, node_id: NodeId| -> bool {
        let node = doc.tree.get(node_id).unwrap();
        for child in node.children() {
            if child.value().is_element() && sel.matches(doc, child.id()) {
                return true;
            }
        }
        false
    }))
}
