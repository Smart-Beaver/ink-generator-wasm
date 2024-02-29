extern crate console_error_panic_hook;

use std::fmt::{Display, Formatter};
use std::panic;
use std::str::FromStr;

use wasm_bindgen::prelude::*;

use generator::source_parser::run;

use crate::code_loader::static_files::with_static_content;

mod logger;
mod code_loader;
pub mod generator;
pub mod prettifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub enum OutputFile {
    Main,
    Data,
    Traits,
    Errors,
    Cargo,
}

impl FromStr for OutputFile {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lib.rs" => Ok(OutputFile::Main),
            "data.rs" => Ok(OutputFile::Data),
            "traits.rs" => Ok(OutputFile::Traits),
            "errors.rs" => Ok(OutputFile::Errors),
            "Cargo.toml" => Ok(OutputFile::Cargo),
            _ => Err(()),
        }
    }
}

impl Display for OutputFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            OutputFile::Main => "lib.rs".to_owned(),
            OutputFile::Data => "data.rs".to_owned(),
            OutputFile::Traits => "traits.rs".to_owned(),
            OutputFile::Errors => "errors.rs".to_owned(),
            OutputFile::Cargo => "Cargo.toml".to_owned(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub enum Standard {
    PSP22,
    PSP34,
}

impl Display for Standard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Standard::PSP22 => "PSP22".fmt(f),
            Standard::PSP34 => "PSP34".fmt(f),
        }
    }
}

pub struct ExternalCrate {
    pub name: &'static str,
    pub version: &'static str,
}

impl Standard {
    pub fn get_external_crate_name(&self) -> Option<ExternalCrate> {
        match self {
            Standard::PSP22 => Some(ExternalCrate {
                name: "psp22-full",
                version: "0.3.0",
            }),
            Standard::PSP34 => Some(ExternalCrate {
                name: "psp34-full",
                version: "0.2.1",
            })
        }
    }
}

impl FromStr for Standard {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PSP22" => Ok(Standard::PSP22),
            "PSP34" => Ok(Standard::PSP34),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct Contract {
    pub standard: Standard,

    #[wasm_bindgen(skip)]
    pub metadata: Option<Metadata>,

    #[wasm_bindgen(skip)]
    pub extensions: Vec<String>,

    #[wasm_bindgen(skip)]
    pub files: Vec<OutputFile>,

    #[wasm_bindgen(skip)]
    pub source: String,

    #[wasm_bindgen(skip)]
    pub license_name: String,

    pub use_external_crate: bool,
}

fn get_all_files() -> Vec<OutputFile> {
    Vec::from([
        OutputFile::Main,
        OutputFile::Data,
        OutputFile::Traits,
        OutputFile::Errors,
        OutputFile::Cargo
    ])
}


#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct Metadata {
    #[wasm_bindgen(skip)]
    pub name: Option<String>,

    #[wasm_bindgen(skip)]
    pub symbol: Option<String>,

    #[wasm_bindgen(skip)]
    pub uri: Option<String>,

    pub decimals: Option<u8>,
}

#[wasm_bindgen]
impl Metadata {
    #[wasm_bindgen(constructor)]
    pub fn new(name: Option<String>, symbol: Option<String>, uri: Option<String>, decimals: Option<u8>) -> Self {
        Self { name, symbol, uri, decimals }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn uri(&self) -> Option<String> {
        self.uri.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> Option<String> {
        self.symbol.clone()
    }
}

#[wasm_bindgen]
impl Contract {
    #[wasm_bindgen(constructor)]
    pub fn new(standard: String, metadata: Option<Metadata>, extensions: js_sys::Array, source: String, license_name: String, use_external_crate: bool, files: Option<js_sys::Array>) -> Contract {
        Self {
            standard: Standard::from_str(&standard).unwrap(),
            metadata,
            extensions: extensions.iter().map(|v| v.as_string().unwrap()).collect(),
            source,
            license_name,
            files: match files {
                Some(unwrapped) => unwrapped.iter().map(|v| {
                    let output_file = v.as_string().unwrap();
                    OutputFile::from_str(&output_file).unwrap()
                }).collect(),
                None => get_all_files(),
            },
            use_external_crate,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> Option<Metadata> {
        self.metadata.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn extensions(&self) -> js_sys::Array {
        self.extensions.iter().map(JsValue::from).collect()
    }

    #[wasm_bindgen(setter)]
    pub fn set_extensions(&mut self, extensions: js_sys::Array) {
        self.extensions = extensions.iter().map(|v| v.as_string().unwrap()).collect();
    }

    #[wasm_bindgen(getter)]
    pub fn files(&self) -> js_sys::Array {
        self.files.iter().map(|output_file| JsValue::from(output_file.to_string())).collect()
    }

    #[wasm_bindgen(setter)]
    pub fn set_files(&mut self, files: Option<js_sys::Array>) {
        self.files = match files {
            Some(unwrapped) => unwrapped.iter().map(|v| {
                let output_file = v.as_string().unwrap();
                OutputFile::from_str(&output_file).unwrap()
            }).collect(),
            None => get_all_files(),
        };
    }

    #[wasm_bindgen(getter)]
    pub fn license_name(&self) -> String {
        self.license_name.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_license_name(&mut self, license_name: String) {
        self.license_name = license_name;
    }
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct MergedFile {
    #[wasm_bindgen(skip)]
    pub name: String,
    #[wasm_bindgen(skip)]
    pub content: String,
}

#[wasm_bindgen]
impl MergedFile {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, content: String) -> Self {
        Self { name, content }
    }

    fn from_js_value(value: JsValue) -> Option<Self> {
        let obj = value.dyn_into::<js_sys::Object>().ok()?;
        let name = js_sys::Reflect::get(&obj, &JsValue::from_str("name")).ok()?;
        let content = js_sys::Reflect::get(&obj, &JsValue::from_str("content")).ok()?;
        Some(MergedFile {
            name: name.as_string()?,
            content: content.as_string()?,
        })
    }
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct ParserResponse {
    #[wasm_bindgen(skip)]
    pub result: bool,

    #[wasm_bindgen(skip)]
    pub message: String,

    #[wasm_bindgen(skip)]
    pub files: Vec<MergedFile>,
}

// Define a struct to represent a file

#[wasm_bindgen]
impl ParserResponse {
    #[wasm_bindgen(constructor)]
    pub fn new(result: bool, message: String, files: js_sys::Array) -> ParserResponse {
        Self { result, message, files: files.iter().filter_map(MergedFile::from_js_value).collect() }
    }

    #[wasm_bindgen(getter)]
    pub fn result(&self) -> bool {
        self.result
    }

    #[wasm_bindgen(setter)]
    pub fn set_result(&mut self, result: bool) {
        self.result = result;
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    #[wasm_bindgen(getter)]
    pub fn files(&self) -> JsValue {
        let files_array = self.files.iter().map(|file| {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(&file.name)).unwrap();
            js_sys::Reflect::set(&obj, &JsValue::from_str("content"), &JsValue::from_str(&file.content)).unwrap();
            JsValue::from(obj)
        }).collect::<js_sys::Array>();

        JsValue::from(files_array)
    }

    #[wasm_bindgen(setter)]
    pub fn set_files(&mut self, files: js_sys::Array) {
        self.files = files.iter().filter_map(MergedFile::from_js_value).collect();
    }
}

#[wasm_bindgen]
pub async fn start(input: Contract) -> ParserResponse {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let standard = input.standard;
    let source = &input.source.clone();
    let license = &input.license_name.clone();
    let files_to_process = match input.use_external_crate {
        true => vec![OutputFile::Main, OutputFile::Cargo], //Generate code that uses external crate - only lib.rs is created
        false => input.files.to_vec()
    };

    match run(input).await {
        Ok(code) => {
            match with_static_content(code, license, source, standard, files_to_process).await {
                Ok(downloaded_files) => ParserResponse {
                    result: true,
                    message: String::new(),
                    files: downloaded_files,
                },
                Err(error) => ParserResponse {
                    result: true,
                    message: error.to_string(),
                    files: Vec::new(),
                }
            }
        }
        Err(error) => ParserResponse {
            result: false,
            message: error.to_string(),
            files: Vec::new(),
        }
    }
}
