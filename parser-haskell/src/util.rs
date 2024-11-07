use lalrpop_util::ParseError;
use lalrpop_util::ParseError::*;

/// Converts the default ParseError shape into something easier to handle.
pub fn simplify_parse_error<'input>(
    error: ParseError<usize, crate::haskell::Token<'input>, &'static str>,
) -> ParseError<usize, String, &'static str> {
    match error {
        InvalidToken { location } => InvalidToken { location },
        UnrecognizedToken {
            token: (start, tok, end),
            expected,
        } => {
            let token = (start, tok.to_string(), end);
            UnrecognizedToken { token, expected }
        }
        ExtraToken {
            token: (start, tok, end),
        } => {
            let token = (start, tok.to_string(), end);
            ExtraToken { token }
        }
        User { error } => User { error },
        UnrecognizedEof { location, expected } => UnrecognizedEof { location, expected },
    }
}

fn code_error(code: &str, tok_pos: usize) {
    let code = format!("\n\n{}", code);
    let code = code.lines().collect::<Vec<_>>();
    let mut pos: isize = 0;
    for (i, lines) in (&code[..]).windows(3).enumerate() {
        if pos + lines[2].len() as isize >= tok_pos as isize {
            let arrow_len = (tok_pos as isize) - (pos - 6);
            let omit_left = if arrow_len > 60 {
                arrow_len as usize - 60
            } else {
                0
            };

            // prints line no. and a 70-char window into line
            let print_line = |n: usize, mut line: &str| {
                if line.len() >= omit_left {
                    line = &line[omit_left..];
                    if line.len() > 70 {
                        line = &line[..70];
                    }
                }
                line = line.trim_end();
                if !line.is_empty() {
                    println!("{:>3} | {}", n, line);
                } else {
                    println!("{:>3} |", n);
                }
            };

            if i > 1 {
                print_line(i - 1, &lines[0]);
            }
            if i > 0 {
                print_line(i, &lines[1]);
            }
            print_line(i + 1, &lines[2]);

            if arrow_len > 0 {
                let arrow_len = arrow_len as usize - omit_left;
                println!("{}^", "~".repeat(arrow_len));
            }
            return;
        }
        pos += (lines[2].len() as isize) + 1;
    }
}

// Print out errors smartly
pub fn print_parse_error(code: &str, err: &ParseError<usize, String, &'static str>) {
    match err {
        ParseError::InvalidToken { location: loc } => {
            println!("Error: Invalid token:");
            code_error(code, *loc);
        }
        ParseError::UnrecognizedToken {
            token: (loc, ref tok, _),
            ..
        } => {
            println!("Error: Unrecognized token `{}`:", tok);
            code_error(code, *loc);
        }
        ParseError::UnrecognizedEof { location, expected } => {
            println!("Error: Unrecognized eof `{}`:", expected.join("\n"));
            code_error(code, *location);
        }
        ParseError::ExtraToken {
            token: (loc, ref tok, _),
        } => {
            println!("Error: Extra token `{}`:", tok);
            code_error(code, *loc);
        }
        _ => (),
    }
}
