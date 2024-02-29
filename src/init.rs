use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use enum_map::{enum_map, EnumMap};
use log::{debug, info};
use once_cell::sync::Lazy;

use ink_generator::{prettifier, Standard};
use ink_generator::generator::{BASE_CONTRACT_FILE_TYPE, CONTRACT_EXTENSION_FILE_TYPE};
use ink_generator::generator::merge::{AstMerger, Merger};
use ink_generator::generator::source_parser::{ExtensionContext, ExtensionKind};

static EXTENSION_PATH_PREFIX: Lazy<EnumMap<ExtensionKind, Option<&'static str>>> = Lazy::new(|| enum_map! {
    ExtensionKind::Ownable => Some("security/"),//Ownable extension is located in security/ directory, so we need to add it to the path
    _ => None,
});

fn get_ast(path: &str) -> syn::File {
    let contract = fs::read_to_string(path).expect("File not found");
    syn::parse_str::<syn::File>(&contract).expect("Failed to parse")
}

fn copy_static(extension: &str, static_content: &str, standard: Standard) {
    write_to_file(
        format!("contracts/{standard}/extensions/tests/{extension}/src/{static_content}.rs"),
        &fs::read_to_string(format!("contracts/{standard}/{static_content}.rs"))
            .expect("File not found"),
    ).expect("Could not write to file.");
}

fn generate(destination: &str, main: &syn::File, extensions: Vec<ExtensionContext>, standard: Standard) {
    info!("Generating tests for: {}", destination);
    let merged = Merger::merge(main, extensions, standard, &None, false).expect("Merge failed");
    let content = prettifier::unparse(&merged);
    let path = format!("contracts/{standard}/extensions/tests/{destination}/src/lib{BASE_CONTRACT_FILE_TYPE}");
    write_to_file(path, &content).expect("Could not write to file.");

    if standard == Standard::PSP34 {
        copy_static(destination, "test_utils", standard);
        copy_static(destination, "unit_tests", standard);
    }

    copy_static(destination, "data", standard);
    copy_static(destination, "errors", standard);
    copy_static(destination, "traits", standard);
}

fn write_to_file<P: AsRef<Path>>(path: P, content: &str) -> io::Result<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Defines extension kind and its dependencies.
/// For example: burnable extension requires ownable extension as well.
type GeneratorExtensionInfo = (ExtensionKind, Vec<ExtensionKind>);

fn generate_test_cases(standard: Standard, extensions: Vec<GeneratorExtensionInfo>) {
    let main_path = format!("contracts/{standard}/lib{}", BASE_CONTRACT_FILE_TYPE);
    let main = get_ast(main_path.as_str());
    debug!("Loading base contract from: {}", main_path);

    for extension in extensions {
        let (kind, dependencies) = extension;
        let dependencies = dependencies.into_iter().map(|kind| {
            let path_prefix = EXTENSION_PATH_PREFIX[kind].unwrap_or("");
            let ast_path = format!("contracts/{standard}/extensions/{path_prefix}{kind}{}", CONTRACT_EXTENSION_FILE_TYPE);
            debug!("Loading extension from: {}", ast_path);
            ExtensionContext {
                kind,
                ast: get_ast(ast_path.as_str()),
            }
        }).collect();
        generate(kind.to_string().as_str(), &main, dependencies, standard);
    }
}


fn generate_psp22_test_cases() {
    generate_test_cases(Standard::PSP22, vec![
        (ExtensionKind::Burnable, vec![ExtensionKind::Burnable, ExtensionKind::Ownable]),
        (ExtensionKind::Mintable, vec![ExtensionKind::Mintable, ExtensionKind::Ownable]),
        (ExtensionKind::Pausable, vec![ExtensionKind::Mintable, ExtensionKind::Pausable, ExtensionKind::Ownable]),
        (ExtensionKind::Capped, vec![ExtensionKind::Mintable, ExtensionKind::Capped, ExtensionKind::Ownable]),
        (ExtensionKind::Wrapper, vec![ExtensionKind::Wrapper]),
    ]);
}

fn generate_psp34_test_cases() {
    generate_test_cases(Standard::PSP34, vec![
        (ExtensionKind::Burnable, vec![ExtensionKind::Burnable, ExtensionKind::Ownable, ExtensionKind::Mintable]),
        (ExtensionKind::Mintable, vec![ExtensionKind::Mintable, ExtensionKind::Ownable]),
        (ExtensionKind::Metadata, vec![ExtensionKind::Metadata, ExtensionKind::Mintable, ExtensionKind::Ownable]),
        (ExtensionKind::Enumerable, vec![ExtensionKind::Enumerable, ExtensionKind::Mintable, ExtensionKind::Ownable]),
    ]);
}

fn main() {
    simple_logger::init().unwrap();
    generate_psp22_test_cases();
    generate_psp34_test_cases();
}
