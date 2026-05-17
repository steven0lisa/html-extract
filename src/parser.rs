/// Command parser - splits command strings with awareness for quotes and commas
pub fn parse_commands(input: &str) -> Result<Vec<String>, String> {
    let mut cmds: Vec<String> = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let max = chars.len();
    let mut last = 0usize;
    let mut next = 0usize;

    loop {
        if next == max {
            if next > last {
                cmds.push(chars[last..next].iter().collect());
            }
            return Ok(cmds);
        }

        let c = chars[next];
        match c {
            ' ' => {
                if next > last {
                    cmds.push(chars[last..next].iter().collect());
                }
                last = next + 1;
            }
            ',' => {
                if next > last {
                    cmds.push(chars[last..next].iter().collect());
                }
                cmds.push(",".to_string());
                last = next + 1;
            }
            '\'' | '"' => {
                let quote_char = c;
                loop {
                    next += 1;
                    if next == max {
                        return Err(format!("Unmatched open quote ({})", quote_char));
                    }
                    if chars[next] == '\\' {
                        next += 1;
                        if next == max {
                            return Err(format!("Unmatched open quote ({})", quote_char));
                        }
                    } else if chars[next] == quote_char {
                        break;
                    }
                }
            }
            _ => {}
        }
        next += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ParseCmdTest {
        input: &'static str,
        expected: Vec<&'static str>,
        ok: bool,
    }

    fn slice_eq(s1: &[String], s2: &[&str]) -> bool {
        if s1.len() != s2.len() {
            return false;
        }
        for i in 0..s1.len() {
            if s1[i] != s2[i] {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_parse_commands() {
        let tests = vec![
            ParseCmdTest { input: "w1 w2", expected: vec!["w1", "w2"], ok: true },
            ParseCmdTest { input: "w1 w2 w3", expected: vec!["w1", "w2", "w3"], ok: true },
            ParseCmdTest { input: "w1 'w2 w3'", expected: vec!["w1", "'w2 w3'"], ok: true },
            ParseCmdTest { input: "w1 \"w2 w3\"", expected: vec!["w1", "\"w2 w3\""], ok: true },
            ParseCmdTest { input: "w1   \"w2 w3\"", expected: vec!["w1", "\"w2 w3\""], ok: true },
            ParseCmdTest { input: "w1   'w2 w3'", expected: vec!["w1", "'w2 w3'"], ok: true },
            ParseCmdTest { input: "w1\"w2 w3\"", expected: vec!["w1\"w2 w3\""], ok: true },
            ParseCmdTest { input: "w1'w2 w3'", expected: vec!["w1'w2 w3'"], ok: true },
            ParseCmdTest { input: "w1\"w2 'w3\"", expected: vec!["w1\"w2 'w3\""], ok: true },
            ParseCmdTest { input: "w1'w2 \"w3'", expected: vec!["w1'w2 \"w3'"], ok: true },
            ParseCmdTest { input: "\"w1 w2\" \"w3\"", expected: vec!["\"w1 w2\"", "\"w3\""], ok: true },
            ParseCmdTest { input: "'w1 w2' \"w3\"", expected: vec!["'w1 w2'", "\"w3\""], ok: true },
            ParseCmdTest { input: "'w1 \\'w2' \"w3\"", expected: vec!["'w1 \\'w2'", "\"w3\""], ok: true },
            ParseCmdTest { input: "'w1 \\'w2 \"w3\"", expected: vec![], ok: false },
            ParseCmdTest { input: "w1 'w2 w3'\"", expected: vec![], ok: false },
            ParseCmdTest { input: "w1 \"w2 w3\"'", expected: vec![], ok: false },
            ParseCmdTest { input: "w1 '  \"w2 w3\"", expected: vec![], ok: false },
            ParseCmdTest { input: "w1 \"  'w2 w3'", expected: vec![], ok: false },
            ParseCmdTest { input: "w1\"w2 w3\"\"", expected: vec![], ok: false },
            ParseCmdTest { input: "w1'w2 w3'''", expected: vec!["w1'w2 w3'''"], ok: true },
            ParseCmdTest { input: "w1\"w2 'w3\"\"", expected: vec![], ok: false },
            ParseCmdTest { input: "w1\"w2 'w3\"\"", expected: vec![], ok: false },
            ParseCmdTest { input: "\"w1 w2\" \"w3\"'", expected: vec![], ok: false },
            ParseCmdTest { input: "'w1 w2' \"w3\"'", expected: vec![], ok: false },
            ParseCmdTest { input: "w1,\"w2 w3\"", expected: vec!["w1", ",", "\"w2 w3\""], ok: true },
            ParseCmdTest { input: "w1,'w2 w3'", expected: vec!["w1", ",", "'w2 w3'"], ok: true },
            ParseCmdTest { input: "w1  ,  \"w2 w3\"", expected: vec!["w1", ",", "\"w2 w3\""], ok: true },
            ParseCmdTest { input: "w1  ,  'w2 w3'", expected: vec!["w1", ",", "'w2 w3'"], ok: true },
            ParseCmdTest { input: "w1,  \"w2 w3\"", expected: vec!["w1", ",", "\"w2 w3\""], ok: true },
            ParseCmdTest { input: "w1,  'w2 w3'", expected: vec!["w1", ",", "'w2 w3'"], ok: true },
            ParseCmdTest { input: "w1  ,\"w2 w3\"", expected: vec!["w1", ",", "\"w2 w3\""], ok: true },
            ParseCmdTest { input: "w1  ,'w2 w3'", expected: vec!["w1", ",", "'w2 w3'"], ok: true },
            ParseCmdTest { input: "w1\"w2, w3\"", expected: vec!["w1\"w2, w3\""], ok: true },
            ParseCmdTest { input: "w1'w2, w3'", expected: vec!["w1'w2, w3'"], ok: true },
            ParseCmdTest { input: "w1\"w2, 'w3\"", expected: vec!["w1\"w2, 'w3\""], ok: true },
            ParseCmdTest { input: "w1'w2, \"w3'", expected: vec!["w1'w2, \"w3'"], ok: true },
            ParseCmdTest { input: "\"w1, w2\" \"w3\"", expected: vec!["\"w1, w2\"", "\"w3\""], ok: true },
            ParseCmdTest { input: "'w1, w2' \"w3\"", expected: vec!["'w1, w2'", "\"w3\""], ok: true },
            ParseCmdTest { input: "'w1, \\'w2' \"w3\"", expected: vec!["'w1, \\'w2'", "\"w3\""], ok: true },
            ParseCmdTest { input: "h1, .article-teaser, .article-content", expected: vec!["h1", ",", ".article-teaser", ",", ".article-content"], ok: true },
            ParseCmdTest { input: "h1 ,.article-teaser ,.article-content", expected: vec!["h1", ",", ".article-teaser", ",", ".article-content"], ok: true },
            ParseCmdTest { input: "h1 , .article-teaser , .article-content", expected: vec!["h1", ",", ".article-teaser", ",", ".article-content"], ok: true },
        ];

        for test in &tests {
            let result = parse_commands(test.input);
            if test.ok {
                match result {
                    Ok(parsed) => {
                        let expected_strings: Vec<String> = test.expected.iter().map(|s| s.to_string()).collect();
                        if !slice_eq(&parsed, &test.expected) {
                            panic!("`{}`: expected {:?}, got {:?}", test.input, expected_strings, parsed);
                        }
                    }
                    Err(e) => {
                        panic!("`{}`: should not error but got: {}", test.input, e);
                    }
                }
            } else {
                if result.is_ok() {
                    panic!("`{}`: should have caused error", test.input);
                }
            }
        }
    }
}
