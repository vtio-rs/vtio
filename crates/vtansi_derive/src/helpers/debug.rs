//! Macro debug helpers

use std::env;
use std::io::{Read, Write};
use std::process::Command;

use proc_macro2::TokenStream;
use syn::DeriveInput;

/// Helper to pretty-print a token stream
pub fn rustfmt_tokens(tokens: &TokenStream) -> String {
    let (stdin_reader, mut stdin_writer) = std::io::pipe()
        .expect("could not create std::io::pipe for rustfmt stdin");
    let (mut stdout_reader, stdout_writer) = std::io::pipe()
        .expect("could not create std::io::pipe for rustfmt stdout");

    let mut rustfmt_cmd = Command::new("rustfmt");
    rustfmt_cmd.stdin(stdin_reader);
    rustfmt_cmd.stdout(stdout_writer);
    let mut rustfmt_child =
        rustfmt_cmd.spawn().expect("rustfmt failed to spawn");

    stdin_writer
        .write_all(tokens.to_string().as_bytes())
        .expect("failed to write to rustfmt stdin");
    drop(stdin_writer);
    drop(rustfmt_cmd);

    let mut output = String::new();
    stdout_reader
        .read_to_string(&mut output)
        .expect("failed to read from rustfmt stdout");

    let status = rustfmt_child.wait().expect("rustfmt failed to start");

    if !status.success() {
        panic!("rustfmt exited with {status}");
    }

    output
}

pub fn debug_print_generated(ast: &DeriveInput, toks: &TokenStream) {
    let debug = env::var("VTANSI_DEBUG");
    if let Ok(s) = debug
        && (s == "1" || ast.ident == s)
    {
        // This intentionally goes into stdout so that
        // we can filter out other rustc output.
        println!("{}", rustfmt_tokens(toks))
    }
}
