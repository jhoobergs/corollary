//! Reformat's Haskell's whitespace to use explicit braces and semicolons.

use crate::trans::encode_literal;
use regex::{Captures, Regex};

// TODO rename "strip comments and other stuff too i guess"
fn strip_comments(text: &str) -> String {
    // Strip comments
    let re = Regex::new(r"\{-[\s\S]*?-\}").unwrap();
    let text = re.replace_all(&text, "").to_string();
    // To prevent double dashes in quotation marks, we trivially ignore quotes
    // that immediately follow the comment.
    // TODO find a better way to do this
    let re = Regex::new(r#"--[^\n\r"][^\n\r]*"#).unwrap();
    let text = re.replace_all(&text, "").to_string();
    let re = Regex::new(r#"--([\n\r])"#).unwrap();
    let text = re.replace_all(&text, "$1").to_string();

    // Strip trailing semicolons (so we don't have "empty statements")
    let re = Regex::new(r"(?m);+\s*$").unwrap();
    let text = re.replace_all(&text, "").to_string();

    // Strip preprocessor decls
    let re = Regex::new(r"(?m)^#(if|ifn?def|endif|else|include|elif).*").unwrap();
    let text = re.replace_all(&text, "").to_string();

    // TODO this should be handled in the parser
    let escape_re = Regex::new(r#"\\([abfnrtv'"\\0]|NUL|ESC)"#).unwrap();
    let decode_escapes = |text: &str| {
        let text = escape_re.replace_all(text, |caps: &Captures| match &caps[1] {
            "a" => "\u{0007}",
            "b" => "\u{0008}",
            "f" => "\u{000C}",
            "n" => "\n",
            "r" => "\r",
            "t" => "\t",
            "v" => "\u{000B}",
            "'" => "'",
            "\"" => "\"",
            "\\" => "\\",
            "0" | "NUL" => "\0",
            "ESC" => "\x1b",
            s => panic!("str escape {}", s),
        });
        text.to_string()
    };

    // Char literals.
    let re = Regex::new(r"'([^'\\]|\\[A-Z]{1,3}|\\.)'").unwrap();
    let text = re
        .replace_all(&text, |caps: &Captures| {
            let v = decode_escapes(&caps[1]);
            assert!(v.len() == 1, "multi char literal {:?}", v);
            format!("'{}'", encode_literal(&v))
        })
        .to_string();

    // Replace all strings with an encoded version to make the parser simpler.
    // If its possible to get LALRPOP to not complain with proper string regexes, should just use
    // that instead
    let re = Regex::new(r#""(([^"\\]|\\.)*?)""#).unwrap();
    let text = re
        .replace_all(&text, |caps: &Captures| {
            let v = decode_escapes(&caps[1]);
            format!("\"{}\"", encode_literal(&v))
        })
        .to_string();

    // Convert forall a b. to forall a b; for simpler parsing
    let re = Regex::new(r"forall\s+(.*?)\.").unwrap();
    let text = re.replace_all(&text, "forall $1;").to_string();

    text
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BlockWord {
    Do,
    Where,
    Of,
    Let,
    In,
    If,
    Then,
    Else,
}

impl BlockWord {
    fn from_str(word: &str) -> Option<Self> {
        use self::BlockWord::*;
        Some(match word {
            "do" => Do,
            "where" => Where,
            "of" => Of,
            "let" => Let,
            "in" => In,
            "if" => If,
            "then" => Then,
            "else" => Else,
            _ => return None,
        })
    }
}

/// Convert indentation to semicolon-delimited brackets, so it can be parsed more easily.
pub fn commify(val: &str) -> String {
    let re_space = Regex::new(r#"^[ \t]+"#).unwrap();
    let re_nl = Regex::new(r#"^\r?\n"#).unwrap();
    let re_word = Regex::new(r#"([\(\{\[\]\}\)]|[^ \t\r\n\(\{\[\]\}\)]+)"#).unwrap();

    // Strip out all comments from the contents.
    let commentless = strip_comments(val);

    // Previous indentation levels
    let mut stash: Vec<(usize, BlockWord)> = vec![];
    // Previously popped from `stash`.
    let mut popped: Option<(usize, BlockWord)> = None;
    // Previous brace nesting levels.
    let mut braces: Vec<isize> = vec![];
    // Previous word was a block starting word.
    let mut trigger: Option<(BlockWord, usize)> = None;
    // How many spaces to indent.
    let mut indent = 0;
    // Check if this is the first word in the line.
    let mut first = true;
    // Check if this is the first word after an in statement.
    let mut in_active = false;

    let mut out = String::new();
    let mut v: &str = &commentless;
    while v.len() > 0 {
        if let Some(cap) = re_space.captures(v) {
            let word = &cap[0];
            out.push_str(word);
            v = &v[word.len()..];

            indent += word.len();
        } else if let Some(cap) = re_nl.captures(v) {
            let word = &cap[0];
            out.push_str(word);
            v = &v[word.len()..];

            indent = 0;
            first = true;
            if stash.len() > 1 {
                for _ in &stash[1..] {
                    out.push_str(" ");
                }
            }
        } else if let Some(cap) = re_word.captures(v) {
            let word = &cap[0];

            macro_rules! pop_brace {
                () => {{
                    popped = stash.pop();
                    braces.pop();
                    out.push_str("}");
                }};
            }

            // TODO why do we need to prevent `;} then` semicolon insertion here
            if first && word != "then" {
                while {
                    if let Some(&(last_level, last_word)) = stash.last() {
                        // Check if we decreased our indent level
                        last_level > indent || (last_level == indent && word == "where")
                    } else {
                        false
                    }
                } {
                    // out.push_str(&format!("[{:?}]", stash.last()));
                    pop_brace!();
                }

                if let Some(&(i, _)) = stash.last() {
                    if i == indent && trigger.is_none() {
                        out.push_str(";");
                    }
                }
            }

            if ["]", ")", "}"].contains(&word) {
                if let Some(brace) = braces.last_mut() {
                    *brace -= 1;
                }
            }
            if ["[", "(", "{"].contains(&word) {
                if let Some(brace) = braces.last_mut() {
                    *brace += 1;
                }
            }

            // End braces insertion when meeting an unbalanced ending ), }, or ]
            while {
                if let Some(brace) = braces.last().map(|x| *x) {
                    brace < 0
                } else {
                    false
                }
            } {
                pop_brace!();
                if braces.len() > 0 {
                    *braces.last_mut().unwrap() -= 1;
                }
            }

            // make sure `let { ... } in` is closed
            if word == "in" && !first {
                // are we still in the `let`?
                if let Some(&(_, BlockWord::Let)) = stash.last() {
                    pop_brace!();
                } else if let Some((_, BlockWord::Let)) = popped {
                    // a `let { ... }` just closed so we don't have to do anything
                } else {
                    let bw = stash.last().expect("`in` at top level").1;
                    out.push_str(&format!(" /* ERR: `in` while in `{:?}` block */ ", bw));
                }
            }

            // make sure `if { ... } then` is closed
            if word == "then" {
                // are we still in the `if`?
                if let Some(&(_, BlockWord::If)) = stash.last() {
                    pop_brace!();
                } else if let Some((_, BlockWord::If)) = popped {
                    // an `If { ... }` just closed so we don't have to do anything
                } else {
                    let bw = stash.last().expect("`if` at top level").1;
                    out.push_str(&format!(" /* ERR: `ifin` while in `{:?}` block */ ", bw));
                }
            }

            out.push_str(word);
            v = &v[word.len()..];

            if let Some((block_word, trigger_indent)) = trigger {
                // The next word after a block word is where the whitespace column begins.
                if first || block_word != BlockWord::In {
                    stash.push((indent, block_word));
                } else {
                    stash.push((trigger_indent, block_word));
                }
            }
            first = false;

            trigger = BlockWord::from_str(word).map(|x| (x, indent));
            if trigger.is_some() {
                out.push_str("{");

                // Trace brace indentation level.
                braces.push(0);
            }

            indent += word.len();
        } else {
            unreachable!("unknown prop {:?}", v);
        }
    }
    for _ in 0..stash.len() {
        out.push_str("}");
    }

    // Replace trailing commas after where statements
    // TODO fix this in the parser instead
    let re = Regex::new(r#"where\s+;"#).unwrap();
    let out = re.replace_all(&out, r#"where "#).to_string();
    // let re = Regex::new(r#"\};\s*where\b"#).unwrap();
    // let out = re.replace_all(&out, r#"} where"#).to_string();
    let re = Regex::new(r#"\};\}"#).unwrap();
    let out = re.replace_all(&out, r#"}}"#).to_string();

    out
}
