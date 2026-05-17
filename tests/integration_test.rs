use std::process::{Command, Stdio};

fn get_pup_binary() -> String {
    std::env::var("CARGO_BIN_EXE_pup-rs").expect("CARGO_BIN_EXE_pup-rs not set")
}

fn read_test_file(name: &str) -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = std::path::Path::new(&manifest_dir).join("tests").join(name);
    std::fs::read_to_string(&path).unwrap()
}

fn run_pup(selector: &str, input: &str, flags: &[&str]) -> String {
    let binary = get_pup_binary();
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
        .expect("Failed to spawn pup-rs");

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(input.as_bytes()).unwrap();
        drop(stdin);
    }

    let output = child.wait_with_output().expect("Failed to wait for pup-rs");
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_basic_tag_selector() {
    let input = read_test_file("index.html");
    let result = run_pup("title", &input, &[]);
    assert!(result.contains("Go"), "Should contain 'Go' in title, got: {}", result);
}

#[test]
fn test_id_selector() {
    let input = read_test_file("index.html");
    let result = run_pup("#footer", &input, &[]);
    assert!(result.contains("footer"), "Should find footer element");
}

#[test]
fn test_class_selector() {
    let input = read_test_file("index.html");
    let result = run_pup(".summary", &input, &[]);
    assert!(!result.is_empty(), "Should find elements with class summary");
}

#[test]
fn test_text_displayer() {
    let input = "<html><body><p>Hello World</p></body></html>";
    let result = run_pup("p text{}", input, &[]);
    assert_eq!(result.trim(), "Hello World");
}

#[test]
fn test_attr_displayer() {
    let input = "<html><body><a href=\"http://example.com\">Link</a></body></html>";
    let result = run_pup("a attr{href}", input, &[]);
    assert_eq!(result.trim(), "http://example.com");
}

#[test]
fn test_json_displayer() {
    let input = "<html><body><a href=\"http://example.com\">Link</a></body></html>";
    let result = run_pup("a json{}", input, &[]);
    assert!(result.contains("\"tag\": \"a\""), "Should contain tag field");
    assert!(result.contains("\"href\": \"http://example.com\""), "Should contain href");
}

#[test]
fn test_number_displayer() {
    let input = "<html><body><p>One</p><p>Two</p><p>Three</p></body></html>";
    let result = run_pup("p", input, &["-n"]);
    assert_eq!(result.trim(), "3");
}

#[test]
fn test_child_selector() {
    let input = "<html><body><div><p>Inside</p></div><p>Outside</p></body></html>";
    let result = run_pup("div > p", input, &[]);
    assert_eq!(result.trim().matches("<p").count(), 1);
}

#[test]
fn test_sibling_selector() {
    let input = "<html><body><p>First</p><span>Second</span></body></html>";
    let result = run_pup("p + span", input, &[]);
    assert!(result.contains("Second"), "Should find span after p");
}

#[test]
fn test_comma_selector() {
    let input = "<html><body><p>Para</p><span>Span</span></body></html>";
    let result = run_pup("p, span", input, &[]);
    assert!(result.contains("Para"));
    assert!(result.contains("Span"));
}

#[test]
fn test_attribute_selector() {
    let input = "<html><body><a href=\"http://example.com\" title=\"Example\">Link</a></body></html>";
    let result = run_pup("a[title=\"Example\"]", input, &[]);
    assert!(result.contains("Link"));
}

#[test]
fn test_pseudo_first_child() {
    let input = "<html><body><div><p>First</p><p>Second</p></div></body></html>";
    let result = run_pup("p:first-child", input, &[]);
    assert!(result.contains("First"));
    assert!(!result.contains("Second"));
}

#[test]
fn test_pseudo_last_child() {
    let input = "<html><body><div><p>First</p><p>Second</p></div></body></html>";
    let result = run_pup("p:last-child", input, &[]);
    assert!(result.contains("Second"));
    assert!(!result.contains("First"));
}

#[test]
fn test_pseudo_nth_child() {
    let input = "<html><body><ul><li>1</li><li>2</li><li>3</li></ul></body></html>";
    let result = run_pup("li:nth-child(2)", input, &[]);
    assert!(result.contains("2"));
    assert!(!result.contains("1") || result.matches("1").count() == 0);
}

#[test]
fn test_pseudo_contains() {
    let input = "<html><body><p>Hello World</p><p>Goodbye</p></body></html>";
    let result = run_pup("p:contains(\"Hello\")", input, &[]);
    assert!(result.contains("Hello"));
    assert!(!result.contains("Goodbye"));
}

#[test]
fn test_text_modifiers() {
    let input = "<html><body><p>Hello World</p></body></html>";
    let result = run_pup("p text{upper}", input, &[]);
    assert_eq!(result.trim(), "HELLO WORLD");

    let result = run_pup("p text{lower}", input, &[]);
    assert_eq!(result.trim(), "hello world");

    let result = run_pup("p text{trim}", input, &[]);
    assert_eq!(result.trim(), "Hello World");
}

#[test]
fn test_raw_output() {
    let input = "<html><body><p>Hello World</p></body></html>";
    let result = run_pup("p", input, &["-r"]);
    // Raw output should not have newlines after tags
    assert!(result.contains("<p>Hello World</p>"), "Raw output should be inline");
}

#[test]
fn test_plain_mode() {
    let input = "<html><body><p>Hello & World</p></body></html>";
    let result = run_pup("p text{}", input, &[]);
    assert!(result.contains("&amp;"), "Should escape HTML by default");

    let result = run_pup("p text{}", input, &["-p"]);
    assert!(result.contains("&") && !result.contains("&amp;"), "Should not escape in plain mode");
}

#[test]
fn test_limit_flag() {
    let input = "<html><body><div><p>One</p><p>Two</p><p>Three</p></div></body></html>";
    let result = run_pup("div", input, &["-l", "1"]);
    assert!(result.contains("..."), "Should contain ellipsis for limited output");
}

#[test]
fn test_indent_flag() {
    let input = "<html><body><div><p>Hello</p></div></body></html>";
    let result = run_pup("div", input, &["-i", "4"]);
    // With 4 spaces indent, we should see 4 spaces before <p>
    assert!(result.contains("    <p>"), "Should use 4-space indent");
}

#[test]
fn test_version_flag() {
    let binary = get_pup_binary();
    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to get version");
    let version = String::from_utf8_lossy(&output.stdout);
    assert!(version.contains("0.4.2"), "Version should be 0.4.2, got: {}", version);
}

#[test]
fn test_file_input() {
    let input = "<html><body><p>FromFile</p></body></html>";
    let temp_path = "/tmp/pup_test_input.html";
    std::fs::write(temp_path, input).unwrap();

    let result = run_pup("p", "", &["-f", temp_path]);
    assert!(result.contains("FromFile"));

    std::fs::remove_file(temp_path).ok();
}
