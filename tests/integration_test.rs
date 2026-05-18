use std::process::{Command, Stdio};

fn get_binary() -> String {
    std::env::var("CARGO_BIN_EXE_html-extract").expect("CARGO_BIN_EXE_html-extract not set")
}

fn read_test_file(name: &str) -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = std::path::Path::new(&manifest_dir).join("tests").join(name);
    std::fs::read_to_string(&path).unwrap()
}

fn run_extract(selector: &str, input: &str, flags: &[&str]) -> String {
    let binary = get_binary();
    let mut cmd = Command::new(&binary);
    for flag in flags {
        cmd.arg(flag);
    }
    cmd.arg(selector);

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn html-extract");

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(input.as_bytes()).unwrap();
        drop(stdin);
    }

    let output = child.wait_with_output().expect("Failed to wait for html-extract");
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_basic_tag_selector() {
    let input = read_test_file("index.html");
    let result = run_extract("title", &input, &[]);
    assert!(result.contains("Go"), "Should contain 'Go' in title, got: {}", result);
}

#[test]
fn test_id_selector() {
    let input = read_test_file("index.html");
    let result = run_extract("#footer", &input, &[]);
    assert!(result.contains("footer"), "Should find footer element");
}

#[test]
fn test_class_selector() {
    let input = read_test_file("index.html");
    let result = run_extract(".summary", &input, &[]);
    assert!(!result.is_empty(), "Should find elements with class summary");
}

#[test]
fn test_text_displayer() {
    let input = "<html><body><p>Hello World</p></body></html>";
    let result = run_extract("p text{}", input, &[]);
    assert_eq!(result.trim(), "Hello World");
}

#[test]
fn test_attr_displayer() {
    let input = "<html><body><a href=\"http://example.com\">Link</a></body></html>";
    let result = run_extract("a attr{href}", input, &[]);
    assert_eq!(result.trim(), "http://example.com");
}

#[test]
fn test_json_displayer() {
    let input = "<html><body><a href=\"http://example.com\">Link</a></body></html>";
    let result = run_extract("a json{}", input, &[]);
    assert!(result.contains("\"tag\": \"a\""), "Should contain tag field");
    assert!(result.contains("\"href\": \"http://example.com\""), "Should contain href");
}

#[test]
fn test_number_displayer() {
    let input = "<html><body><p>One</p><p>Two</p><p>Three</p></body></html>";
    let result = run_extract("p", input, &["-n"]);
    assert_eq!(result.trim(), "3");
}

#[test]
fn test_child_selector() {
    let input = "<html><body><div><p>Inside</p></div><p>Outside</p></body></html>";
    let result = run_extract("div > p", input, &[]);
    assert_eq!(result.trim().matches("<p").count(), 1);
}

#[test]
fn test_sibling_selector() {
    let input = "<html><body><p>First</p><span>Second</span></body></html>";
    let result = run_extract("p + span", input, &[]);
    assert!(result.contains("Second"), "Should find span after p");
}

#[test]
fn test_comma_selector() {
    let input = "<html><body><p>Para</p><span>Span</span></body></html>";
    let result = run_extract("p, span", input, &[]);
    assert!(result.contains("Para"));
    assert!(result.contains("Span"));
}

#[test]
fn test_attribute_selector() {
    let input = "<html><body><a href=\"http://example.com\" title=\"Example\">Link</a></body></html>";
    let result = run_extract("a[title=\"Example\"]", input, &[]);
    assert!(result.contains("Link"));
}

#[test]
fn test_pseudo_first_child() {
    let input = "<html><body><div><p>First</p><p>Second</p></div></body></html>";
    let result = run_extract("p:first-child", input, &[]);
    assert!(result.contains("First"));
    assert!(!result.contains("Second"));
}

#[test]
fn test_pseudo_last_child() {
    let input = "<html><body><div><p>First</p><p>Second</p></div></body></html>";
    let result = run_extract("p:last-child", input, &[]);
    assert!(result.contains("Second"));
    assert!(!result.contains("First"));
}

#[test]
fn test_pseudo_nth_child() {
    let input = "<html><body><ul><li>1</li><li>2</li><li>3</li></ul></body></html>";
    let result = run_extract("li:nth-child(2)", input, &[]);
    assert!(result.contains("2"));
    assert!(!result.contains("1") || result.matches("1").count() == 0);
}

#[test]
fn test_pseudo_contains() {
    let input = "<html><body><p>Hello World</p><p>Goodbye</p></body></html>";
    let result = run_extract("p:contains(\"Hello\")", input, &[]);
    assert!(result.contains("Hello"));
    assert!(!result.contains("Goodbye"));
}

#[test]
fn test_text_modifiers() {
    let input = "<html><body><p>Hello World</p></body></html>";
    let result = run_extract("p text{upper}", input, &[]);
    assert_eq!(result.trim(), "HELLO WORLD");

    let result = run_extract("p text{lower}", input, &[]);
    assert_eq!(result.trim(), "hello world");

    let result = run_extract("p text{trim}", input, &[]);
    assert_eq!(result.trim(), "Hello World");
}

#[test]
fn test_raw_output() {
    let input = "<html><body><p>Hello World</p></body></html>";
    let result = run_extract("p", input, &["-r"]);
    assert!(result.contains("<p>Hello World</p>"), "Raw output should be inline");
}

#[test]
fn test_plain_mode() {
    let input = "<html><body><p>Hello & World</p></body></html>";
    let result = run_extract("p text{}", input, &[]);
    assert!(result.contains("&amp;"), "Should escape HTML by default");

    let result = run_extract("p text{}", input, &["-p"]);
    assert!(result.contains("&") && !result.contains("&amp;"), "Should not escape in plain mode");
}

#[test]
fn test_limit_flag() {
    let input = "<html><body><div><p>One</p><p>Two</p><p>Three</p></div></body></html>";
    let result = run_extract("div", input, &["-l", "1"]);
    assert!(result.contains("..."), "Should contain ellipsis for limited output");
}

#[test]
fn test_indent_flag() {
    let input = "<html><body><div><p>Hello</p></div></body></html>";
    let result = run_extract("div", input, &["-i", "4"]);
    assert!(result.contains("    <p>"), "Should use 4-space indent");
}

#[test]
fn test_version_flag() {
    let binary = get_binary();
    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to get version");
    let version = String::from_utf8_lossy(&output.stdout);
    assert!(version.contains("0.1.0"), "Version should be 0.1.0, got: {}", version);
}

#[test]
fn test_file_input() {
    let input = "<html><body><p>FromFile</p></body></html>";
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("html_extract_test_input.html");
    std::fs::write(&temp_path, input).unwrap();

    let result = run_extract("p", "", &["-f", temp_path.to_str().unwrap()]);
    assert!(result.contains("FromFile"));

    std::fs::remove_file(&temp_path).ok();
}

// --- Slim mode tests ---

#[test]
fn test_slim_basic_filtering() {
    let input = r#"<html><body><div id="main" class="container"><p>Hello</p><span class="highlight">World</span></div></body></html>"#;
    let result = run_extract("", input, &["--slim"]);
    // Only elements with id/class should appear, no tag prefix
    assert!(result.contains("#main.container"), "Should contain #main.container, got: {}", result);
    assert!(result.contains(".highlight"), "Should contain .highlight, got: {}", result);
    // <p> has no id/class, should not appear
    assert!(!result.contains("Hello"), "Plain <p> content should not appear");
}

#[test]
fn test_slim_no_tag_prefix() {
    let input = r#"<html><body><div class="a"><ul class="b"><li class="c"></li></ul><table class="d"></table></div></body></html>"#;
    let result = run_extract("", input, &["--slim"]);
    // No tag prefix at all — just identifiers
    assert!(result.contains(".a"), "Should have .a without tag prefix");
    assert!(result.contains(".b"), "Should have .b without tag prefix");
    assert!(result.contains(".c"), "Should have .c without tag prefix");
    assert!(result.contains(".d"), "Should have .d without tag prefix");
    assert!(!result.contains("d.a"), "Should NOT have d.a tag prefix");
    assert!(!result.contains("u.b"), "Should NOT have u.b tag prefix");
}

#[test]
fn test_slim_attrs_output() {
    let input = r#"<html><body><input type="text" name="q" value="" data-role="search"><button type="submit" class="btn primary">Go</button></body></html>"#;
    let result = run_extract("", input, &["--slim"]);
    // No tag prefix, just attrs
    assert!(result.contains("[data-role=search]"), "Should contain data-role attr, got: {}", result);
    assert!(result.contains("[name=q]"), "Should contain name attr, got: {}", result);
    assert!(result.contains("[type=text]"), "Should contain type attr, got: {}", result);
    assert!(result.contains("[value=]"), "Should contain empty value attr, got: {}", result);
    assert!(result.contains(".btn.primary"), "Should have classes without tag prefix, got: {}", result);
    assert!(result.contains("[type=submit]"), "Should contain submit type, got: {}", result);
    assert!(!result.contains("input["), "Should NOT have input tag prefix");
    // btn appears inside .btn.primary class string, so we check for standalone "btn["
    assert!(!result.contains("btn["), "Should NOT have btn tag prefix before brackets");
}

#[test]
fn test_slim_indent_no_container() {
    let input = r#"<html><body><div><div><span class="deep">text</span></div></div></body></html>"#;
    let result = run_extract("", input, &["--slim"]);
    // The two wrapping divs have no id/class, so .deep should be at level 0
    let lines: Vec<&str> = result.trim().lines().collect();
    assert_eq!(lines.len(), 1, "Should have exactly 1 line, got: {:?}", lines);
    assert_eq!(lines[0], ".deep", "Should be at root level with no indent");
}

#[test]
fn test_slim_with_selector() {
    let input = r#"<html><body><div id="main" class="container"><div class="article"><a class="link">Link</a></div></div></body></html>"#;
    let result = run_extract("div.article", input, &["--slim"]);
    // Should only show content within div.article
    assert!(result.contains(".link"), "Should contain .link within article");
    assert!(!result.contains("#main"), "Should not contain elements outside selector scope");
}

#[test]
fn test_slim_empty_output() {
    let input = r#"<html><body><p>Just text</p><div><span>more</span></div></body></html>"#;
    let result = run_extract("", input, &["--slim"]);
    assert!(result.trim().is_empty(), "Should output nothing when no elements have id/class, got: {}", result);
}

#[test]
fn test_slim_indent_level() {
    let input = r#"<html><body><div id="main"><div class="sub"><span class="text"></span></div></div></body></html>"#;
    let result = run_extract("", input, &["--slim", "-i", "2"]);
    let lines: Vec<&str> = result.trim().lines().collect();
    assert_eq!(lines.len(), 3, "Should have 3 lines, got: {:?}", lines);
    assert_eq!(lines[0], "#main");
    assert!(lines[1].starts_with("  .sub"), "Second line should be indented with 2 spaces, got: '{}'", lines[1]);
    assert!(lines[2].starts_with("    .text"), "Third line should be indented with 4 spaces, got: '{}'", lines[2]);
}
