use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use ink_generator::{prettifier, Standard};
use ink_generator::generator::merge::{AstMerger, Merger};
use ink_generator::generator::source_parser::{ExtensionContext, ExtensionKind};

fn get_ast(path: &str) -> syn::File {
    let contract = fs::read_to_string(path).expect("File not found");
    syn::parse_str::<syn::File>(&contract).expect("Failed to parse")
}

fn copy_static(extension: &str, static_content: &str) {
    write_to_file(
        format!("contracts/PSP22/extensions/tests/{extension}/src/{static_content}.rs"),
        &fs::read_to_string(format!("contracts/PSP22/{static_content}.rs"))
            .expect("File not found"),
    ).expect("Could not write to file.");
}

fn generate(destination: &str, main: &syn::File, extensions: Vec<ExtensionContext>) {
    let merged = Merger::merge(main, extensions, Standard::PSP22, &None, false).expect("Merge failed");
    let content = prettifier::unparse(&merged);
    let path = format!("contracts/PSP22/extensions/tests/{destination}/src/lib.rs");
    write_to_file(path, &content).expect("Could not write to file.");
    copy_static(destination, "data");
    copy_static(destination, "errors");
    copy_static(destination, "traits");
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

fn main() {
    let main = get_ast("contracts/PSP22/lib.rs");

    let burnable = ExtensionContext {
        kind: ExtensionKind::Burnable,
        ast: get_ast("contracts/PSP22/extensions/burnable.trs"),
    };

    let mintable = ExtensionContext {
        kind: ExtensionKind::Mintable,
        ast: get_ast("contracts/PSP22/extensions/mintable.trs"),
    };

    let pausable = ExtensionContext {
        kind: ExtensionKind::Pausable,
        ast: get_ast("contracts/PSP22/extensions/pausable.trs"),
    };

    let capped = ExtensionContext {
        kind: ExtensionKind::Capped,
        ast: get_ast("contracts/PSP22/extensions/capped.trs"),
    };

    let wrapper = ExtensionContext {
        kind: ExtensionKind::Wrapper,
        ast: get_ast("contracts/PSP22/extensions/wrapper.trs"),
    };

    let ownable = ExtensionContext {
        kind: ExtensionKind::Ownable,
        ast: get_ast("contracts/PSP22/extensions/security/ownable.trs"),
    };

    generate("burnable", &main, Vec::from([burnable, ownable.clone()]));
    generate("mintable", &main, Vec::from([mintable.clone(), ownable.clone()]));
    generate("pausable", &main, Vec::from([mintable.clone(), pausable, ownable.clone()]));
    generate("capped", &main, Vec::from([mintable.clone(), capped, ownable]));
    generate("wrapper", &main, Vec::from([wrapper]));
}
