use syn::File;

/// Converts a Rust syntax tree back into source code, with some formatting adjustments.
///
/// This function takes a syntax tree represented by a `syn::File` and converts it back into
/// a formatted Rust source code string. It removes documentation comments and adds newlines
/// before specific keywords (`impl`, `#`, and `fn`) for readability, provided they follow a
/// line ending with a '}' or ';'.
///
/// The method first unparses the syntax tree using `prettyplease::unparse` to convert it into
/// a string, then processes each line to filter out documentation comments and adjust formatting.
///
/// # Arguments
///
/// * `file` - A reference to a `syn::File` representing the Rust syntax tree.
///
/// # Returns
///
/// A `String` containing the formatted Rust source code.
///
/// # Examples
///
/// ```
/// use syn::parse_file;
///
/// let src = "fn main() { println!(\"Hello, world!\"); }";
/// let syntax_tree = parse_file(src).unwrap();
/// let formatted_src = ink_generator::prettifier::unparse(&syntax_tree);
/// assert_eq!(formatted_src, "fn main() {\n    println!(\"Hello, world!\");\n}");
/// ```
pub fn unparse(file: &File) -> String {
    let input = &prettyplease::unparse(file);

    let mut result = String::new();
    let binding = doc_comments_remove(input);
    let lines: Vec<&str> = binding.lines().collect();

    for (key, line) in lines.iter().enumerate() {
        let current = line.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        let previous = if key > 0 {
            lines[key - 1].chars().filter(|c| !c.is_whitespace()).collect::<String>()
        } else {
            String::new()
        };

        for tag in &["impl", "#", "fn"] {
            if !previous.is_empty()
                && current.starts_with(tag)
                && (previous.ends_with('}') || previous.ends_with(';'))
            {
                result.push('\n');
            }
        }

        result.push('\n');
        result.push_str(line);
    }

    result
}

/// Removes documentation comments from a Rust source code string.
///
/// This function processes each line of the input string and filters out
/// lines that are recognized as Rust documentation comments. Both inner
/// (`//!`) and outer (`///`) doc comments are removed. Lines are considered
/// doc comments if they start with `///` or `//!`, possibly preceded by
/// whitespace. Other lines, including code and non-doc comment lines, are
/// preserved.
///
/// # Arguments
///
/// * `input` - A string slice containing Rust source code.
///
/// # Returns
///
/// A `String` with doc comments removed.
///
/// # Examples
///
/// ```
/// let code = r#"//! Module comment
/// /// Function comment
/// fn example() -> i32 {
///     /// Inside function
///     42
/// }"#;
/// let cleaned_code = ink_generator::prettifier::doc_comments_remove(code);
/// assert_eq!(cleaned_code, "fn example() -> i32 {\n    42\n}");
pub fn doc_comments_remove(input: &str) -> String {
    input
        .lines()
        .filter(|line| {
            let trimmed_line = line.trim_start();
            !trimmed_line.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
