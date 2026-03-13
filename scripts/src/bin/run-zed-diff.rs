// Reads a Zed diff file and prints the diff entries
// Rust is fun

#![allow(dead_code, unused)]

use std::collections::HashMap;
use std::path::Path;
use std::{env, fs};

use regex::{Regex, RegexBuilder};

const DIFF_REGEX_PATTERN: &str = r#"
    \*\*Tool\sCall:\s([^*]+)\*\*\n  # 1: Relative file path
    Status:\sCompleted\n+
    Diff:\s(.+)$\n                  # 2: Absolute file path
    ```\n
    (?s:(.+?))\n                    # 3: Diff content
    ```
"#;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut dif_file_path: Option<String> = None;
    let mut list_files: bool = false;
    let mut output_content: bool = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            // Diff file path
            "-d" | "--diff" => {
                if i + 1 < args.len() {
                    dif_file_path = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Error: --diff option requires a file path");
                    return;
                }
            }

            // List files in diff
            "-l" | "--list" => {
                list_files = true;
            }

            // Output diff content
            "-o" | "--output" => {
                output_content = true;
            }

            // Unk
            _ => {
                eprintln!("Error: unknown option: {}", args[i]);
                return;
            }
        }

        i += 1;
    }

    // Validate diff file path
    if dif_file_path.is_none() {
        eprintln!("Error: --diff option is required");
        return;
    }

    // Read diff file and normalize line endings
    let dif_file_path = dif_file_path.unwrap();
    let diff_file_content = fs::read_to_string(&dif_file_path)
        .expect("Failed to read diff file")
        .replace("\r\n", "\n");

    // Parse diff file, should panic if anything
    let diff_file = parse_diff_file(&dif_file_path, &diff_file_content);
    // println!("{:?}", diff_file);

    // Handle options
    if list_files {
        handle_list_files(&diff_file);
    }

    if output_content {
        handle_output_content(&diff_file);
    }
}

/// Prints the absolute paths of file entries in the diff file
fn handle_list_files(diff_file: &DiffFile<'_>) {
    println!("Files in diff:");

    for (abs_path, diff_entry) in diff_file.entries.iter() {
        println!("\t{} ({})", abs_path, diff_entry.relative_path);
    }
}

/// Creates an output folder for the diff file and writes the diff entries to files
fn handle_output_content(diff_file: &DiffFile<'_>) {
    println!("Outputting diff content to files...");

    let output_folder_dir = env::current_dir()
        .unwrap()
        .join(format!("zed_diff_output/{}", diff_file.file_name()));
    fs::create_dir_all(&output_folder_dir).unwrap();

    for diff_entry in diff_file.entries.values() {
        let output_file_path = output_folder_dir.join(diff_entry.relative_path);

        // Create parent dir if it doesn't exist
        let output_dir = output_file_path.parent().unwrap();
        if !output_dir.exists() {
            fs::create_dir_all(output_dir).unwrap();
        }

        // Write diff entry to file
        fs::write(output_file_path, diff_entry.diff).unwrap();
    }

    println!("Output complete.");
}

/// Single diff file entry
#[derive(Debug)]
struct DiffEntry<'a> {
    relative_path: &'a str, // more lifetimes practice for me :P
    absolute_path: &'a str,
    diff: &'a str,
}

/// Diff file containing multiple entries
#[derive(Debug)]
struct DiffFile<'a> {
    entries: HashMap<&'a str, DiffEntry<'a>>, // absolute path -> entry
    file_path: &'a str,
}

impl DiffFile<'_> {
    fn file_name(&self) -> &str {
        let path = Path::new(self.file_path);
        path.file_stem().and_then(|n| n.to_str()).unwrap_or("")
    }
}

/// Parses a diff file into a [`DiffFile`] struct
fn parse_diff_file<'a>(diff_file_path: &'a str, diff_file: &'a str) -> DiffFile<'a> {
    let mut entries = HashMap::new();

    let re = RegexBuilder::new(DIFF_REGEX_PATTERN)
        .multi_line(true)
        .ignore_whitespace(true)
        .build()
        .unwrap();

    for c in re.captures_iter(diff_file) {
        let relative_path = c
            .get(1)
            .map(|m| m.as_str())
            .expect("Relative path not found");

        let absolute_path = c
            .get(2)
            .map(|m| m.as_str())
            .expect("Absolute path not found");

        let diff = c.get(3).map(|m| m.as_str()).expect("Diff not found");

        entries.insert(
            absolute_path,
            DiffEntry {
                relative_path,
                absolute_path,
                diff,
            },
        );
    }

    DiffFile {
        entries,
        file_path: diff_file_path,
    }
}
