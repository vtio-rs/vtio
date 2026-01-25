//! Build script to parse rgb.txt and generate a named color lookup table using PHF.

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("named_colors.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let rgb_txt_path = Path::new(&manifest_dir).join("rgb.txt");

    println!("cargo:rerun-if-changed=rgb.txt");
    println!("cargo:rerun-if-changed=build.rs");

    let file = File::open(&rgb_txt_path).expect("Failed to open rgb.txt");
    let reader = BufReader::new(file);

    // Use a HashMap to deduplicate colors by normalized name (lowercase, no spaces)
    // We keep the first occurrence of each normalized name
    let mut colors: HashMap<String, (u8, u8, u8)> = HashMap::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('!') || line.starts_with('#') {
            continue;
        }

        // Parse format: "R G B\t\tname" or "R G B  name"
        // The RGB values are right-aligned in columns
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let r: u8 = match parts[0].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let g: u8 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let b: u8 = match parts[2].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        // The name is everything after the RGB values
        let name = parts[3..].join(" ");

        // Normalize the name for lookup: lowercase, no spaces
        let normalized = name.to_lowercase().replace(' ', "");

        // Only insert if we haven't seen this normalized name before
        colors.entry(normalized).or_insert((r, g, b));
    }

    // Generate the output file
    let out_file =
        File::create(&dest_path).expect("Failed to create output file");
    let mut writer = BufWriter::new(out_file);

    writeln!(writer, "// Auto-generated from rgb.txt - do not edit!").unwrap();
    writeln!(writer).unwrap();
    writeln!(writer, "/// Number of named colors in the lookup table.")
        .unwrap();
    writeln!(
        writer,
        "pub const NAMED_COLOR_COUNT: usize = {};",
        colors.len()
    )
    .unwrap();
    writeln!(writer).unwrap();

    // Build PHF map with byte slice keys
    let mut phf_builder = phf_codegen::Map::<&[u8]>::new();
    for (normalized, (r, g, b)) in &colors {
        phf_builder.entry(normalized.as_bytes(), &format!("({r}, {g}, {b})"));
    }

    writeln!(
        writer,
        "/// Perfect hash map of named X11 colors (normalized name bytes -> (r, g, b))."
    )
    .unwrap();
    writeln!(
        writer,
        "static NAMED_COLORS_MAP: phf::Map<&'static [u8], (u8, u8, u8)> = {};",
        phf_builder.build()
    )
    .unwrap();
    writeln!(writer).unwrap();

    // Direct lookup for pre-normalized byte slices (no allocation)
    writeln!(
        writer,
        "/// Look up a named color by pre-normalized name bytes (lowercase ASCII, no spaces)."
    )
    .unwrap();
    writeln!(writer, "/// Returns (r, g, b) as 8-bit values if found.")
        .unwrap();
    writeln!(writer, "#[inline]").unwrap();
    writeln!(
        writer,
        "pub fn lookup_normalized(name: &[u8]) -> Option<(u8, u8, u8)> {{"
    )
    .unwrap();
    writeln!(writer, "    NAMED_COLORS_MAP.get(name).copied()").unwrap();
    writeln!(writer, "}}").unwrap();
    writeln!(writer).unwrap();

    // Convenience function that normalizes the input
    writeln!(
        writer,
        "/// Look up a named color by name (case-insensitive, spaces ignored)."
    )
    .unwrap();
    writeln!(writer, "/// Returns (r, g, b) as 8-bit values if found.")
        .unwrap();
    writeln!(writer, "#[inline]").unwrap();
    writeln!(
        writer,
        "pub fn lookup_named_color(name: &str) -> Option<(u8, u8, u8)> {{"
    )
    .unwrap();
    writeln!(writer, "    let normalized: Vec<u8> = name.bytes()").unwrap();
    writeln!(writer, "        .filter(|b| !b.is_ascii_whitespace())").unwrap();
    writeln!(writer, "        .map(|b| b.to_ascii_lowercase())").unwrap();
    writeln!(writer, "        .collect();").unwrap();
    writeln!(writer).unwrap();
    writeln!(writer, "    lookup_normalized(&normalized)").unwrap();
    writeln!(writer, "}}").unwrap();
}
