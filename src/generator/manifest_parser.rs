use toml_edit::{Document, value};
/// Updates the given `Cargo.toml` string with the provided license information.
///
/// This function takes a string representation of a `Cargo.toml` file, along with optional
/// license information. It updates the corresponding field in the `Cargo.toml`
/// if the provided value is `Some`. Fields are left unchanged if the value is `None`.
///
/// # Arguments
///
/// * `cargo_toml` - A string that holds the contents of a `cargo.toml` file.
/// * `license` - An optional string representing the license.
///
/// # Returns
///
/// This function returns a `String` which is the modified `Cargo.toml` content. If parsing or
/// serialization of TOML data fails, the function will panic with an appropriate error message.
///
/// # Examples
///
/// ```
/// let cargo_toml = r#"
/// [package]
/// name = "your_package"
/// version = "0.1.0"
/// "#.to_owned();
///
/// let updated_cargo_toml = ink_generator::generator::manifest_parser::add_author_and_license(
///     cargo_toml,
///     Some("MIT".to_owned()),
/// );
/// println!("{}", updated_cargo_toml);
/// ```
pub fn add_author_and_license(
    cargo_toml: String,
    license: Option<String>,
) -> String {
    let mut parsed_toml = cargo_toml.parse::<Document>().expect("Unable to parse TOML");
    if let Some(license) = license {
        parsed_toml["package"]["license"] = value(license);
    }

    parsed_toml.to_string()
}
