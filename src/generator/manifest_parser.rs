use toml_edit::{Document, value};

use crate::ExternalCrate;

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
fn add_author_and_license(
    mut parsed_toml: Document,
    license: Option<String>,
) -> Document {
    if let Some(license) = license {
        parsed_toml["package"]["license"] = value(license);
    }

    parsed_toml
}

fn add_std_features(
    mut parsed_toml: Document,
    std_features: Option<String>,
) -> Document {
    if let Some(std_features) = std_features {
        if let Some(std_array) = parsed_toml["features"]["std"].as_array_mut() {
            std_array.push(format!("{}/std", std_features));
        }
    }

    parsed_toml
}

fn add_crate_import(
    mut parsed_toml: Document,
    external_crate: Option<ExternalCrate>,
) -> Document {
    if let Some(external_crate) = external_crate {
        parsed_toml["dependencies"][external_crate.name]["version"] = value(external_crate.version);
        parsed_toml["dependencies"][external_crate.name]["default-features"] = value(false);
    }

    parsed_toml
}

pub fn update_cargo_config(
    cargo_toml: String,
    license: Option<String>,
    external_crate: Option<ExternalCrate>,
) -> String {
    cargo_toml.parse::<Document>()
        .map(|doc| add_author_and_license(doc, license))
        .map(|doc| add_std_features(doc, external_crate.as_ref().map(|ext| String::from(ext.name))))
        .map(|doc| add_crate_import(doc, external_crate))
        .map(|doc| doc.to_string())
        .expect("Unable to parse TOML")
}
