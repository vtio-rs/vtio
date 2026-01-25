//! Parser profiling program for benchmarking TerminalInputParser performance.
//!
//! This program generates various terminal input patterns and feeds them
//! through the parser repeatedly to measure performance under different
//! workloads.
//!
//! # Usage
//!
//! ```sh
//! # Run all benchmarks (default)
//! cargo bench --bench parser
//!
//! # Run specific sizes
//! cargo bench --bench parser -- --size small
//! cargo bench --bench parser -- --size medium --size large
//! cargo bench --bench parser -- -s small -s large
//!
//! # Run specific input kinds
//! cargo bench --bench parser -- --kind ascii
//! cargo bench --bench parser -- --kind unicode --kind ansi
//! cargo bench --bench parser -- -k csi -k mouse
//!
//! # Run specific parsers
//! cargo bench --bench parser -- --parser vtio
//! cargo bench --bench parser -- --parser termwiz
//! cargo bench --bench parser -- -p vtio -p termwiz
//!
//! # Combine size, kind, and parser filters
//! cargo bench --bench parser -- --size large --kind ascii --parser vtio
//!
//! # Show help
//! cargo bench --bench parser -- --help
//! ```

use std::collections::HashSet;
use std::hint::black_box;
use std::io::Write;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::time::{Duration, Instant};

use ansi_parser::AnsiParser;
use crossterm::event::{poll as crossterm_poll, read as crossterm_read};
use nix::pty::openpty;
use nix::unistd::dup2;
use termion::event::parse_event as termion_parse_event;
use termwiz::input::InputParser as TermwizInputParser;
use vte::{Parser as VteParser, Perform};
use vtio::parser::TerminalInputParser;
use vtparse::{VTActor, VTParser};

/// Buffer size category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Size {
    Small,
    Medium,
    Large,
}

impl Size {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "small" | "s" => Some(Size::Small),
            "medium" | "m" => Some(Size::Medium),
            "large" | "l" => Some(Size::Large),
            _ => None,
        }
    }

    fn all() -> HashSet<Size> {
        [Size::Small, Size::Medium, Size::Large]
            .into_iter()
            .collect()
    }

    fn buffer_size(&self) -> usize {
        match self {
            Size::Small => 1024,
            Size::Medium => 64 * 1024,
            Size::Large => 1024 * 1024,
        }
    }

    fn iterations(&self) -> usize {
        match self {
            Size::Small => 100_000,
            Size::Medium => 10_000,
            Size::Large => 1_000,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Size::Small => "1KB",
            Size::Medium => "64KB",
            Size::Large => "1MB",
        }
    }
}

/// Input data kind category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    Ascii,
    Mixed,
    Unicode,
    Control,
    Ansi,
    Csi,
    Mouse,
    Pathological,
}

impl Kind {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ascii" | "plain" => Some(Kind::Ascii),
            "mixed" | "mixedcase" => Some(Kind::Mixed),
            "unicode" | "utf8" => Some(Kind::Unicode),
            "control" | "ctrl" => Some(Kind::Control),
            "ansi" => Some(Kind::Ansi),
            "csi" => Some(Kind::Csi),
            "mouse" => Some(Kind::Mouse),
            "pathological" | "path" => Some(Kind::Pathological),
            _ => None,
        }
    }

    fn all() -> HashSet<Kind> {
        [
            Kind::Ascii,
            Kind::Mixed,
            Kind::Unicode,
            Kind::Control,
            Kind::Ansi,
            Kind::Csi,
            Kind::Mouse,
            Kind::Pathological,
        ]
        .into_iter()
        .collect()
    }

    fn label(&self) -> &'static str {
        match self {
            Kind::Ascii => "Plain ASCII",
            Kind::Mixed => "Mixed case",
            Kind::Unicode => "Unicode text",
            Kind::Control => "Control chars",
            Kind::Ansi => "ANSI sequences",
            Kind::Csi => "CSI sequences",
            Kind::Mouse => "Mouse events",
            Kind::Pathological => "Pathological",
        }
    }

    fn generate(&self, size: usize) -> Vec<u8> {
        match self {
            Kind::Ascii => generate_plain_text(size),
            Kind::Mixed => generate_mixed_case(size),
            Kind::Unicode => generate_unicode_text(size),
            Kind::Control => generate_control_chars(size),
            Kind::Ansi => generate_ansi_sequences(size),
            Kind::Csi => generate_csi_sequences(size),
            Kind::Mouse => generate_mouse_events(size),
            Kind::Pathological => generate_pathological(size),
        }
    }
}

/// Parser implementation to benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names)]
enum Parser {
    // Input parsers (terminal input: keyboard, mouse, etc.)
    Vtio,
    Termwiz,
    Crossterm,
    Termion,
    // Output parsers (terminal output: cursor, colors, escape sequences)
    Vte,
    Vtparse,
    AnsiParser,
}

impl Parser {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "vtio" | "v" => Some(Parser::Vtio),
            "termwiz" | "tw" | "t" => Some(Parser::Termwiz),
            "crossterm" | "ct" | "cross" => Some(Parser::Crossterm),
            "termion" | "ti" => Some(Parser::Termion),
            "vte" => Some(Parser::Vte),
            "vtparse" | "vtp" => Some(Parser::Vtparse),
            "ansi-parser" | "ansi" | "ap" => Some(Parser::AnsiParser),
            "all" => None, // Handled separately via --all-parsers flag
            _ => None,
        }
    }

    fn all() -> HashSet<Parser> {
        [
            Parser::Vtio,
            Parser::Termwiz,
            Parser::Crossterm,
            Parser::Termion,
            Parser::Vte,
            Parser::Vtparse,
            Parser::AnsiParser,
        ]
        .into_iter()
        .collect()
    }

    fn all_input() -> HashSet<Parser> {
        [
            Parser::Vtio,
            Parser::Termwiz,
            Parser::Crossterm,
            Parser::Termion,
        ]
        .into_iter()
        .collect()
    }

    fn all_output() -> HashSet<Parser> {
        [Parser::Vte, Parser::Vtparse, Parser::AnsiParser]
            .into_iter()
            .collect()
    }

    fn label(&self) -> &'static str {
        match self {
            Parser::Vtio => "vtio",
            Parser::Termwiz => "termwiz",
            Parser::Crossterm => "crossterm",
            Parser::Termion => "termion",
            Parser::Vte => "vte",
            Parser::Vtparse => "vtparse",
            Parser::AnsiParser => "ansi-parser",
        }
    }
}

/// A minimal VTActor implementation for vtparse benchmarking.
struct VtparseActor;

impl VTActor for VtparseActor {
    fn print(&mut self, c: char) {
        black_box(c);
    }

    fn execute_c0_or_c1(&mut self, control: u8) {
        black_box(control);
    }

    fn dcs_hook(
        &mut self,
        mode: u8,
        params: &[i64],
        intermediates: &[u8],
        ignored_excess_intermediates: bool,
    ) {
        black_box((mode, params, intermediates, ignored_excess_intermediates));
    }

    fn dcs_put(&mut self, byte: u8) {
        black_box(byte);
    }

    fn dcs_unhook(&mut self) {}

    fn esc_dispatch(
        &mut self,
        params: &[i64],
        intermediates: &[u8],
        ignored_excess_intermediates: bool,
        byte: u8,
    ) {
        black_box((params, intermediates, ignored_excess_intermediates, byte));
    }

    fn csi_dispatch(
        &mut self,
        params: &[vtparse::CsiParam],
        parameters_truncated: bool,
        byte: u8,
    ) {
        black_box((params, parameters_truncated, byte));
    }

    fn osc_dispatch(&mut self, params: &[&[u8]]) {
        black_box(params);
    }

    fn apc_dispatch(&mut self, data: Vec<u8>) {
        black_box(data);
    }
}

/// A minimal Perform implementation for vte benchmarking.
struct VtePerformer;

impl Perform for VtePerformer {
    fn print(&mut self, c: char) {
        std::hint::black_box(c);
    }

    fn execute(&mut self, byte: u8) {
        std::hint::black_box(byte);
    }

    fn hook(
        &mut self,
        params: &vte::Params,
        intermediates: &[u8],
        ignore: bool,
        action: char,
    ) {
        std::hint::black_box((params, intermediates, ignore, action));
    }

    fn put(&mut self, byte: u8) {
        std::hint::black_box(byte);
    }

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        std::hint::black_box((params, bell_terminated));
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        intermediates: &[u8],
        ignore: bool,
        action: char,
    ) {
        std::hint::black_box((params, intermediates, ignore, action));
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        std::hint::black_box((intermediates, ignore, byte));
    }
}

/// Benchmark configuration.
struct BenchConfig {
    name: String,
    parser: Parser,
    iterations: usize,
    data: Vec<u8>,
}

impl BenchConfig {
    fn new(size: Size, kind: Kind, parser: Parser) -> Self {
        let name =
            format!("{} ({}) [{}]", kind.label(), size.label(), parser.label());
        let buffer_size = size.buffer_size();
        let iterations = size.iterations();
        let data = kind.generate(buffer_size);

        Self {
            name,
            parser,
            iterations,
            data,
        }
    }
}

/// Command-line options.
struct Options {
    sizes: HashSet<Size>,
    kinds: HashSet<Kind>,
    parsers: HashSet<Parser>,
}

impl Options {
    fn parse() -> Result<Self, String> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut sizes = HashSet::new();
        let mut kinds = HashSet::new();
        let mut parsers = HashSet::new();
        let mut all_parsers = false;
        let mut all_input = false;
        let mut all_output = false;

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];

            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "--all-parsers" | "-a" => {
                    all_parsers = true;
                }
                "--all-input" => {
                    all_input = true;
                }
                "--all-output" => {
                    all_output = true;
                }
                "-s" | "--size" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("--size requires a value".to_string());
                    }
                    match Size::from_str(&args[i]) {
                        Some(size) => {
                            sizes.insert(size);
                        }
                        None => {
                            return Err(format!(
                                "invalid size '{}'. Valid options: small, medium, large",
                                args[i]
                            ));
                        }
                    }
                }
                "-k" | "--kind" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("--kind requires a value".to_string());
                    }
                    match Kind::from_str(&args[i]) {
                        Some(kind) => {
                            kinds.insert(kind);
                        }
                        None => {
                            return Err(format!(
                                "invalid kind '{}'. Valid options: ascii, mixed, unicode, control, ansi, csi, mouse, pathological",
                                args[i]
                            ));
                        }
                    }
                }
                "-p" | "--parser" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("--parser requires a value".to_string());
                    }
                    match Parser::from_str(&args[i]) {
                        Some(parser) => {
                            parsers.insert(parser);
                        }
                        None => {
                            return Err(format!(
                                "invalid parser '{}'. Valid options: vtio, termwiz, vte, termion, vtparse, ansi-parser",
                                args[i]
                            ));
                        }
                    }
                }
                _ if arg.starts_with("--size=") => {
                    let value = &arg[7..];
                    match Size::from_str(value) {
                        Some(size) => {
                            sizes.insert(size);
                        }
                        None => {
                            return Err(format!(
                                "invalid size '{value}'. Valid options: small, medium, large"
                            ));
                        }
                    }
                }
                _ if arg.starts_with("--kind=") => {
                    let value = &arg[7..];
                    match Kind::from_str(value) {
                        Some(kind) => {
                            kinds.insert(kind);
                        }
                        None => {
                            return Err(format!(
                                "invalid kind '{value}'. Valid options: ascii, mixed, unicode, control, ansi, csi, mouse, pathological"
                            ));
                        }
                    }
                }
                _ if arg.starts_with("--parser=") => {
                    let value = &arg[9..];
                    match Parser::from_str(value) {
                        Some(parser) => {
                            parsers.insert(parser);
                        }
                        None => {
                            return Err(format!(
                                "invalid parser '{value}'. Valid options: vtio, termwiz, vte, termion, vtparse, ansi-parser"
                            ));
                        }
                    }
                }
                _ => {
                    // Ignore unknown arguments (cargo may pass some)
                }
            }
            i += 1;
        }

        // Default to all if none specified
        if sizes.is_empty() {
            sizes = Size::all();
        }
        if kinds.is_empty() {
            kinds = Kind::all();
        }
        if all_parsers {
            parsers = Parser::all();
        } else if all_input {
            parsers = Parser::all_input();
        } else if all_output {
            parsers = Parser::all_output();
        } else if parsers.is_empty() {
            parsers.insert(Parser::Vtio);
        }

        Ok(Self {
            sizes,
            kinds,
            parsers,
        })
    }
}

fn print_help() {
    eprintln!(
        r#"vtio parser profiler

USAGE:
    cargo bench --bench parser -- [OPTIONS]

OPTIONS:
    -h, --help              Print this help message
    -s, --size <SIZE>       Buffer size to test (can be specified multiple times)
                            Values: small (1KB), medium (64KB), large (1MB)
    -k, --kind <KIND>       Input kind to test (can be specified multiple times)
                            Values: ascii, mixed, unicode, control, ansi, csi, mouse, pathological
    -p, --parser <PARSER>   Parser implementation to test (can be specified multiple times)
                            Input parsers: vtio (default), termwiz, crossterm, termion
                            Output parsers: vte, vtparse, ansi-parser
    -a, --all-parsers       Test all parser implementations
    --all-input             Test all input parsers (vtio, termwiz, crossterm, termion)
    --all-output            Test all output parsers (vte, vtparse, ansi-parser)

EXAMPLES:
    # Run all benchmarks (default)
    cargo bench --bench parser

    # Run only small buffer benchmarks
    cargo bench --bench parser -- --size small

    # Run small and large buffer benchmarks
    cargo bench --bench parser -- -s small -s large

    # Run only ASCII and ANSI input kinds
    cargo bench --bench parser -- --kind ascii --kind ansi

    # Run only vtio parser
    cargo bench --bench parser -- --parser vtio

    # Compare all parsers
    cargo bench --bench parser -- --all-parsers

    # Compare vtio and termwiz on large ASCII input
    cargo bench --bench parser -- --size large --kind ascii -p vtio -p termwiz

    # Combine filters
    cargo bench --bench parser -- --size large --kind ascii --kind unicode
"#
    );
}

/// Run a single benchmark for vtio parser and return timing information.
fn run_benchmark_vtio(config: &BenchConfig) -> Duration {
    let start = Instant::now();

    for _ in 0..config.iterations {
        let mut parser = TerminalInputParser::new();
        let data = black_box(&config.data);

        parser.feed_with(data, &mut |event| {
            black_box(event);
        });
    }

    start.elapsed()
}

/// Run a single benchmark for termwiz parser and return timing information.
fn run_benchmark_termwiz(config: &BenchConfig) -> Duration {
    let start = Instant::now();

    for _ in 0..config.iterations {
        let mut parser = TermwizInputParser::new();
        let data = black_box(&config.data);

        parser.parse(
            data,
            |event| {
                black_box(event);
            },
            false,
        );
    }

    start.elapsed()
}

/// Run a single benchmark for vte parser and return timing information.
fn run_benchmark_vte(config: &BenchConfig) -> Duration {
    let start = Instant::now();

    for _ in 0..config.iterations {
        let mut parser = VteParser::new();
        let mut performer = VtePerformer;
        let data = black_box(&config.data);

        parser.advance(&mut performer, data);
    }

    start.elapsed()
}

/// Run a single benchmark for termion parser and return timing information.
fn run_benchmark_termion(config: &BenchConfig) -> Duration {
    let start = Instant::now();

    for _ in 0..config.iterations {
        let data = black_box(&config.data);
        let mut iter = data.iter().map(|&b| Ok(b));

        while let Some(Ok(byte)) = iter.next() {
            if let Ok(event) = termion_parse_event(byte, &mut iter) {
                black_box(event);
            }
        }
    }

    start.elapsed()
}

/// Run a single benchmark for vtparse parser and return timing information.
fn run_benchmark_vtparse(config: &BenchConfig) -> Duration {
    let start = Instant::now();

    for _ in 0..config.iterations {
        let mut parser = VTParser::new();
        let mut actor = VtparseActor;
        let data = black_box(&config.data);

        parser.parse(data, &mut actor);
    }

    start.elapsed()
}

/// Run a single benchmark for ansi-parser and return timing information.
fn run_benchmark_ansi_parser(config: &BenchConfig) -> Duration {
    let start = Instant::now();

    for _ in 0..config.iterations {
        let data = black_box(&config.data);
        // ansi-parser works on &str, so we need to convert
        // Use from_utf8_lossy for invalid UTF-8 sequences
        let text = String::from_utf8_lossy(data);

        for output in text.ansi_parse() {
            black_box(output);
        }
    }

    start.elapsed()
}

/// Run a single benchmark for crossterm parser and return timing information.
///
/// Crossterm doesn't expose a public parser API, so we use a PTY to feed data
/// to crossterm's event reading system. This adds some I/O overhead but provides
/// a realistic comparison of crossterm's parsing performance.
fn run_benchmark_crossterm(config: &BenchConfig) -> Duration {
    // Create a PTY pair - we'll write to master and crossterm reads from slave
    let pty = openpty(None, None).expect("Failed to create PTY");

    // Save original stdin so we can restore it later
    let original_stdin =
        unsafe { OwnedFd::from_raw_fd(libc::dup(libc::STDIN_FILENO)) };

    // Redirect stdin to the slave side of the PTY
    dup2(pty.slave.as_raw_fd(), libc::STDIN_FILENO)
        .expect("Failed to dup2 slave to stdin");

    // Create a writer for the master side
    let mut master =
        unsafe { std::fs::File::from_raw_fd(pty.master.as_raw_fd()) };
    std::mem::forget(pty.master); // Don't close it when pty goes out of scope

    let start = Instant::now();

    for _ in 0..config.iterations {
        let data = black_box(&config.data);

        // Write all data to the PTY master
        master.write_all(data).expect("Failed to write to PTY");
        master.flush().expect("Failed to flush PTY");

        // Read events until no more are available
        while crossterm_poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(event) = crossterm_read() {
                black_box(event);
            }
        }
    }

    let elapsed = start.elapsed();

    // Restore original stdin
    dup2(original_stdin.as_raw_fd(), libc::STDIN_FILENO)
        .expect("Failed to restore stdin");

    elapsed
}

/// Run a benchmark based on the parser type.
fn run_benchmark(config: &BenchConfig) -> Duration {
    match config.parser {
        Parser::Vtio => run_benchmark_vtio(config),
        Parser::Termwiz => run_benchmark_termwiz(config),
        Parser::Vte => run_benchmark_vte(config),
        Parser::Termion => run_benchmark_termion(config),
        Parser::Vtparse => run_benchmark_vtparse(config),
        Parser::AnsiParser => run_benchmark_ansi_parser(config),
        Parser::Crossterm => run_benchmark_crossterm(config),
    }
}

/// Generate plain ASCII text.
fn generate_plain_text(size: usize) -> Vec<u8> {
    let text = "The quick brown fox jumps over the lazy dog. ";
    text.as_bytes().iter().cycle().take(size).copied().collect()
}

/// Generate text with mixed case.
fn generate_mixed_case(size: usize) -> Vec<u8> {
    let text = "HeLLo WoRLd! ThIs Is MiXeD CaSe TeXt. ";
    text.as_bytes().iter().cycle().take(size).copied().collect()
}

/// Generate text with Unicode characters.
fn generate_unicode_text(size: usize) -> Vec<u8> {
    let text = "Hello 世界! 🦀 Rust ñ café Ω α β γ. ";
    text.as_bytes().iter().cycle().take(size).copied().collect()
}

/// Generate text with ANSI escape sequences.
fn generate_ansi_sequences(size: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let patterns = [
        b"\x1b[1;31mRed\x1b[0m ".as_slice(),
        b"\x1b[32mGreen\x1b[0m ".as_slice(),
        b"\x1b[1;34mBlue\x1b[0m ".as_slice(),
        b"\x1b[H\x1b[2J".as_slice(),
        b"\x1b[10;20HCursor ".as_slice(),
    ];

    let mut idx = 0;
    while result.len() < size {
        result.extend_from_slice(patterns[idx % patterns.len()]);
        idx += 1;
    }

    result.truncate(size);
    result
}

/// Generate control characters mixed with text.
fn generate_control_chars(size: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let text = b"Hello\r\n\tWorld\x08\x1b";

    while result.len() < size {
        result.extend_from_slice(text);
    }

    result.truncate(size);
    result
}

/// Generate CSI sequences (cursor movement, colors, etc).
fn generate_csi_sequences(size: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let sequences = [
        b"\x1b[A".as_slice(),                 // Up
        b"\x1b[B".as_slice(),                 // Down
        b"\x1b[C".as_slice(),                 // Forward
        b"\x1b[D".as_slice(),                 // Back
        b"\x1b[H".as_slice(),                 // Home
        b"\x1b[2J".as_slice(),                // Clear screen
        b"\x1b[38;5;208m".as_slice(),         // 256 color
        b"\x1b[48;2;100;150;200m".as_slice(), // RGB color
    ];

    let mut idx = 0;
    while result.len() < size {
        result.extend_from_slice(sequences[idx % sequences.len()]);
        result.extend_from_slice(b"text ");
        idx += 1;
    }

    result.truncate(size);
    result
}

/// Generate mouse event sequences.
fn generate_mouse_events(size: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let events = [
        b"\x1b[<0;10;20M".as_slice(),  // Mouse down
        b"\x1b[<0;10;20m".as_slice(),  // Mouse up
        b"\x1b[<32;15;25M".as_slice(), // Mouse drag
        b"\x1b[<64;5;5M".as_slice(),   // Scroll up
        b"\x1b[<65;5;5M".as_slice(),   // Scroll down
    ];

    let mut idx = 0;
    while result.len() < size {
        result.extend_from_slice(events[idx % events.len()]);
        result.extend_from_slice(b"abc ");
        idx += 1;
    }

    result.truncate(size);
    result
}

/// Generate worst-case scenario: many incomplete sequences.
fn generate_pathological(size: usize) -> Vec<u8> {
    let mut result = Vec::new();

    while result.len() < size {
        result.push(b'\x1b');
        result.push(b'[');
        result.extend_from_slice(b"1;2;3;4;5");
        result.push(b'm');
        result.push(b'x');
    }

    result.truncate(size);
    result
}

fn main() {
    let options = match Options::parse() {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("Run with --help for usage information.");
            std::process::exit(1);
        }
    };

    println!("vtio parser profile");
    println!("===================\n");

    // Print active filters
    let sizes: Vec<_> = {
        let mut s: Vec<_> = options.sizes.iter().collect();
        s.sort_by_key(|s| match s {
            Size::Small => 0,
            Size::Medium => 1,
            Size::Large => 2,
        });
        s
    };
    let kinds: Vec<_> = {
        let mut k: Vec<_> = options.kinds.iter().collect();
        k.sort_by_key(|k| match k {
            Kind::Ascii => 0,
            Kind::Mixed => 1,
            Kind::Unicode => 2,
            Kind::Control => 3,
            Kind::Ansi => 4,
            Kind::Csi => 5,
            Kind::Mouse => 6,
            Kind::Pathological => 7,
        });
        k
    };
    let parsers: Vec<_> = {
        let mut p: Vec<_> = options.parsers.iter().collect();
        p.sort_by_key(|p| match p {
            Parser::Vtio => 0,
            Parser::Termwiz => 1,
            Parser::Crossterm => 2,
            Parser::Termion => 3,
            Parser::Vte => 4,
            Parser::Vtparse => 5,
            Parser::AnsiParser => 6,
        });
        p
    };

    println!(
        "Sizes: {}",
        sizes
            .iter()
            .map(|s| s.label())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "Kinds: {}",
        kinds
            .iter()
            .map(|k| k.label())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "Parsers: {}\n",
        parsers
            .iter()
            .map(|p| p.label())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Build benchmark configurations based on filters
    let mut benchmarks = Vec::new();

    for &size in &sizes {
        for &kind in &kinds {
            for &parser in &parsers {
                benchmarks.push(BenchConfig::new(*size, *kind, *parser));
            }
        }
    }

    // Track results for comparison summary
    let mut results: Vec<(String, Parser, f64)> = Vec::new();

    for config in &benchmarks {
        let elapsed = run_benchmark(config);
        let total_bytes = config.data.len() * config.iterations;
        let throughput_mbs =
            (total_bytes as f64 / 1_000_000.0) / elapsed.as_secs_f64();

        println!(
            "{:<35} {:>8} iters  {:>8.2} ms  {:>10.2} MB/s",
            config.name,
            config.iterations,
            elapsed.as_secs_f64() * 1000.0,
            throughput_mbs
        );

        // Store for comparison (extract base name without parser suffix)
        let base_name = config
            .name
            .rsplit_once(" [")
            .map(|(s, _)| s.to_string())
            .unwrap_or(config.name.clone());
        results.push((base_name, config.parser, throughput_mbs));
    }

    // Print comparison summary if multiple parsers were tested
    let tested_parsers: Vec<_> = options.parsers.iter().copied().collect();
    if tested_parsers.len() > 1 && options.parsers.contains(&Parser::Vtio) {
        println!("\n---");
        println!("Comparison Summary (vtio vs others):");
        println!("---\n");

        // Group results by base name and parser
        let mut parser_results: std::collections::HashMap<
            String,
            std::collections::HashMap<Parser, f64>,
        > = std::collections::HashMap::new();

        for (name, parser, throughput) in &results {
            parser_results
                .entry(name.clone())
                .or_default()
                .insert(*parser, *throughput);
        }

        // Collect and sort benchmark names
        let mut bench_names: Vec<_> = parser_results.keys().cloned().collect();
        bench_names.sort();

        // Print comparisons for each benchmark
        for name in &bench_names {
            let results_for_bench = &parser_results[name];
            if let Some(&vtio_tp) = results_for_bench.get(&Parser::Vtio) {
                let mut parts: Vec<String> =
                    vec![format!("vtio: {:>10.2} MB/s", vtio_tp)];

                for parser in &[
                    Parser::Termwiz,
                    Parser::Crossterm,
                    Parser::Termion,
                    Parser::Vte,
                    Parser::Vtparse,
                    Parser::AnsiParser,
                ] {
                    if let Some(&other_tp) = results_for_bench.get(parser) {
                        let ratio = vtio_tp / other_tp;
                        parts.push(format!(
                            "{}: {:>10.2} MB/s",
                            parser.label(),
                            other_tp
                        ));
                        if ratio >= 1.0 {
                            parts.push(format!("(vtio {ratio:.1}x faster)"));
                        } else {
                            parts.push(format!(
                                "({} {:.1}x faster)",
                                parser.label(),
                                1.0 / ratio
                            ));
                        }
                    }
                }

                println!("{:<25} {}", name, parts.join("  "));
            }
        }

        // Overall averages vs each parser
        println!();
        for other_parser in &[
            Parser::Termwiz,
            Parser::Crossterm,
            Parser::Termion,
            Parser::Vte,
            Parser::Vtparse,
            Parser::AnsiParser,
        ] {
            if !options.parsers.contains(other_parser) {
                continue;
            }

            let ratios: Vec<f64> = parser_results
                .values()
                .filter_map(|bench_results| {
                    let vtio_tp = bench_results.get(&Parser::Vtio)?;
                    let other_tp = bench_results.get(other_parser)?;
                    Some(vtio_tp / other_tp)
                })
                .collect();

            if !ratios.is_empty() {
                let avg_ratio: f64 =
                    ratios.iter().sum::<f64>() / ratios.len() as f64;
                println!(
                    "Average vs {}: vtio is {:.2}x {}",
                    other_parser.label(),
                    if avg_ratio >= 1.0 {
                        avg_ratio
                    } else {
                        1.0 / avg_ratio
                    },
                    if avg_ratio >= 1.0 { "faster" } else { "slower" }
                );
            }
        }
    }

    println!("\nProfile complete!");
}
