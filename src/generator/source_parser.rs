use std::error::Error;
use std::str::FromStr;

use futures::future::join_all;

use crate::{Contract, prettifier};
use crate::code_loader::loader::load_source;
use crate::generator::merge::{AstMerger, Merger};
use crate::logger::console_log;
use crate::logger::log;


const BASE_CONTRACT_FILE_TYPE: &str = ".rs";

const CONTRACT_EXTENSION_FILE_TYPE: &str = ".trs";

#[derive(Clone)]
pub struct ExtensionContext {
    pub kind: ExtensionKind,
    pub ast: syn::File,
}

#[derive(Clone)]
pub enum ExtensionKind {
    Metadata,
    Mintable,
    Burnable,
    Wrapper,
    FlashMint,
    Pausable,
    Capped,
    Batch,
    Enumerable,
    Ownable,
    AccessControl,
}

impl FromStr for ExtensionKind {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "metadata" => Ok(ExtensionKind::Metadata),
            "mintable" => Ok(ExtensionKind::Mintable),
            "burnable" => Ok(ExtensionKind::Burnable),
            "wrapper" => Ok(ExtensionKind::Wrapper),
            "flash_mint" => Ok(ExtensionKind::FlashMint),
            "pausable" => Ok(ExtensionKind::Pausable),
            "capped" => Ok(ExtensionKind::Capped),
            "batch" => Ok(ExtensionKind::Batch),
            "enumerable" => Ok(ExtensionKind::Enumerable),
            "security/ownable" => Ok(ExtensionKind::Ownable),
            "security/access_control" => Ok(ExtensionKind::AccessControl),
            _ => Err(format!("Unknown extension: {}", s).into()),
        }
    }
}


async fn load_parse_ast(filepath: &str) -> Result<syn::File, Box<dyn Error>> {
    let code_string = load_source(filepath).await?;
    Ok(syn::parse_str(&code_string)?)
}

async fn load_base_contract(source: &str, standard: &str) -> Result<syn::File, Box<dyn Error>> {
    console_log!("Loading base contract[{standard}]: lib{BASE_CONTRACT_FILE_TYPE}");
    load_parse_ast(&format!("{source}/{standard}/lib{BASE_CONTRACT_FILE_TYPE}")).await
}

async fn load_extension(extension: &str, source: &str, standard: &str) -> Result<syn::File, Box<dyn Error>> {
    console_log!("Loading extension[{standard}]: {extension}");
    load_parse_ast(&format!("{source}/{standard}/extensions/{extension}{CONTRACT_EXTENSION_FILE_TYPE}")).await
}

pub async fn run(contract: Contract) -> Result<String, Box<dyn Error>> {
    console_log!("Running parser for contract: {:#?}", contract);
    let standard = &contract.standard.to_string();
    let source = &contract.source;
    let base_contract_ast = load_base_contract(&contract.source, standard).await?;

    let extensions = join_all(
        contract.extensions.iter()
            .map(|extension| (extension, source, standard))
            .map(|(extension, source, standard)| async move {
                (extension, load_extension(extension, source, standard).await)
            })
    ).await;

    //Ignore not loaded extensions
    let mut extensions_checked: Vec<ExtensionContext> = Vec::new();
    for (extension_name, load_result) in extensions {
        extensions_checked.push(ExtensionContext {
            kind: ExtensionKind::from_str(extension_name)?,
            ast: load_result?,
        });
    }

    let merger = Merger::merge(
        &base_contract_ast,
        extensions_checked,
        contract.standard,
        &contract.metadata,
    )?;

    Ok(prettifier::unparse(&merger))
}
