mod display;
mod parser;
mod selector;

use std::io::Read;

use clap::Parser;

use parser::parse_commands;
use selector::{SelectorFunc, SelectorFuncType, build_selector_func};



pub const VERSION: &str = "0.4.2";

/// Command line tool for processing HTML using CSS selectors
#[derive(Parser, Debug)]
#[command(name = "pup", version = VERSION, about = "pup is a command line tool for processing HTML")]
struct Args {
    /// Print result with color
    #[arg(short = 'c', long = "color")]
    color: bool,

    /// File to read from
    #[arg(short = 'f', long = "file")]
    file: Option<String>,

    /// Number of spaces to use for indent or character
    #[arg(short = 'i', long = "indent")]
    indent: Option<String>,

    /// Print number of elements selected
    #[arg(short = 'n', long = "number")]
    number: bool,

    /// Restrict number of levels printed
    #[arg(short = 'l', long = "limit")]
    limit: Option<i32>,

    /// Don't escape html
    #[arg(short = 'p', long = "plain")]
    plain: bool,

    /// Raw output
    #[arg(short = 'r', long = "raw")]
    raw: bool,

    /// Ignore non-standard HTML tags
    #[arg(short = 's', long = "strict")]
    strict: bool,

    /// Preserve preformatted text
    #[arg(long = "pre")]
    pre: bool,

    /// Specify the charset
    #[arg(long = "charset")]
    charset: Option<String>,

    /// Selectors and optional display function
    selectors: Vec<String>,
}

pub struct Config {
    pub color: bool,
    pub escape_html: bool,
    pub raw: bool,
    pub strict: bool,
    pub pre: bool,
    pub max_print_level: i32,
    pub indent_string: String,
    pub charset: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            color: false,
            escape_html: true,
            raw: false,
            strict: false,
            pre: false,
            max_print_level: -1,
            indent_string: " ".to_string(),
            charset: None,
        }
    }
}

fn main() {
    let args = Args::parse();

    let mut config = Config::default();
    config.color = args.color;
    config.escape_html = !args.plain;
    config.raw = args.raw;
    config.strict = args.strict;
    config.pre = args.pre;
    config.charset = args.charset;

    if let Some(limit) = args.limit {
        config.max_print_level = limit;
    }

    if let Some(indent) = args.indent {
        if let Ok(n) = indent.parse::<usize>() {
            config.indent_string = " ".repeat(n);
        } else {
            config.indent_string = indent;
        }
    }

    let number_mode = args.number;

    // Read HTML input
    let html_input = if let Some(ref filename) = args.file {
        match std::fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(2);
            }
        }
    } else {
        let mut input = String::new();
        match std::io::stdin().read_to_string(&mut input) {
            Ok(_) => input,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(2);
            }
        }
    };

    // Parse HTML
    let document = match selector::parse_html(&html_input) {
        Ok(doc) => doc,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    };

    // Parse the selectors
    let cmds_string = args.selectors.join(" ");
    let cmds = match parse_commands(&cmds_string) {
        Ok(cmds) => cmds,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    };

    // Parse selectors and build selector functions
    let mut selector_funcs: Vec<Option<SelectorFunc>> = vec![];
    let mut func_type = SelectorFuncType::Descendant;
    let mut last_cmd_is_displayer = false;

    for (i, cmd) in cmds.iter().enumerate() {
        let is_last = i == cmds.len() - 1;
        if is_last {
            if display::parse_displayer(cmd).is_ok() {
                last_cmd_is_displayer = true;
                continue;
            }
        }
        match cmd.as_str() {
            "*" => continue,
            ">" => func_type = SelectorFuncType::Child,
            "+" => func_type = SelectorFuncType::NextSibling,
            "," => selector_funcs.push(None),
            _ => {
                let sel_func = match build_selector_func(cmd, func_type, config.strict) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Selector parsing error: {}", e);
                        std::process::exit(2);
                    }
                };
                selector_funcs.push(Some(sel_func));
                func_type = SelectorFuncType::Descendant;
            }
        }
    }

    // Execute selectors
    let root_id = document.tree.root().id();
    let mut selected_nodes: Vec<ego_tree::NodeId> = vec![];
    let mut curr_nodes: Vec<ego_tree::NodeId> = vec![root_id];

    for selector_func in &selector_funcs {
        if selector_func.is_none() {
            selected_nodes.extend(curr_nodes.iter().cloned());
            curr_nodes = vec![root_id];
        } else {
            curr_nodes = selector_func.as_ref().unwrap()(&document, &curr_nodes);
        }
    }
    selected_nodes.extend(curr_nodes.iter().cloned());

    // Display results
    if number_mode {
        println!("{}", selected_nodes.len());
    } else {
        let displayer = display::create_displayer(&cmds, last_cmd_is_displayer);
        displayer.display(&document, &selected_nodes, &config);
    }
}
