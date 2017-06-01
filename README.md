# Porting Corrode to Rust

The goal of this project is to make [Corrode, the C to Rust translator written in Haskell](https://github.com/jameysharp/corrode), and Language-C, the Haskell library and C AST it depends on, extensible by a more applicable audience: Rust developers. It aims to broaden development access to a critical piece of Rust's tooling and compelling draw: its C to Rust conversion tool.

To accomplish this goal, this project has developed a compiler (Corollary) than can partially convert these libraries into Rust, letting contributors help by manually completing the conversion into idiomatic Rust code. Much like conversion process for Corrode itself!

This project has incomplete, but very extensive, automated ports of Language-C and Corrode. Read on for how we can get to 100% working crates.

```
git clone http://github.com/tcr/corrode-but-in-rust --recursive
```

These are the crates contained in this repo:

* **`parser-haskell/`**, an original Haskell Parser written in LALRPOP.
* **`corollary/`**, an experimental Haskell to Rust compiler. This is used to generate (part of) the `parser-c` and `rust-corrode` crates.
* **`corollary-support/`**, a support crate for converted Haskell code to use.
* **`parser-c/`**, a largely automatically cross-compiled version of the `language-c` Haskell module for parsing C code.
* **`rust-corrode/`**, a largely automatically cross-compiled Rust port of the Corrode program for converting C into Rust code.

Sound wild? Here is the plan:

- This project contains a proof-of-concept cross-compiler from Haskell to Rust (Corollary) which is not designed to be either correct or generalizable; instead, it's tailored to port these two libraries.
- The crates `rust-corrode/` and `rust-language-c/` both contain a `compile-haskell.sh` script which semi cross-compiles its code using the above crate. As files are ported over, they can be finalized (fully ported) and removed from `compile-haskell.sh` until no files remain.
- If a problem can be fixed in the conversion instead of manually, edit **corollary** to add more support for the Haskell into Rust conversion. [Follow along in this tracking issue!](https://github.com/tcr/corrode-but-in-rust/issues/1)

One big caveat: Parser.x and Lexer.y, two files for lexing and parsing used by Language-C, are written using Haskell tools and in Haskell. Because their converted files are very regularly generated, we for now just opt to have these conversions done as a separate build pipeline for the Language-c port (`parser-c/`). A unique requirement for Corollary: modifications to these Haskell files should convert to Rust without issues, unlike other files (which can be manually edited to cross the cap to correctness.)

## References

* [Ten Things You Should Know About Haskell Syntax](https://www.fpcomplete.com/blog/2012/09/ten-things-you-should-know-about-haskell-syntax)
* [Haskell: The Confusing Parts](http://echo.rsmw.net/n00bfaq.html)

## License

This project licensed as MIT or Apache-2, at your option.

Language-C licensed as BSD-3.

Corrode licensed as GPLv2.
