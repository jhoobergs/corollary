#[macro_use]
extern crate errln;

extern crate base64;
extern crate lalrpop_util;
extern crate regex;

pub mod ast;

lalrpop_util::lalrpop_mod!(haskell);

pub mod trans;
pub mod util;
pub mod whitespace;

use whitespace::commify;

/// Preprocess code to remove comments and convert whitepsace to brace blocks.
/// TODO: merge this into parse() below once result lifetimes can be worked out
pub fn preprocess(input: &str) -> String {
    commify(input)
}

/// Entry point for parsing modules
pub fn parse<'input, 'err>(
    errors: &'err mut Vec<lalrpop_util::ErrorRecovery<usize, (usize, &'input str), ()>>,
    input: &'input str,
) -> Result<ast::Module, lalrpop_util::ParseError<usize, crate::haskell::Token<'input>, &'static str>>
{
    crate::haskell::ModuleParser::new().parse(errors, input)
}
