use std::error::Error;
use crate::code_loader::loader::load_source;
use crate::{get_all_files, MergedFile, OutputFile};
use crate::generator::manifest_parser::add_author_and_license;
use crate::prettifier::doc_comments_remove;

async fn download_static_content(source: &str, standard: &str, file_string: &str) -> String {
    match load_source(&format!("{source}/{standard}/{file_string}")).await {
        Ok(content) => content,
        Err(_e) => "".to_owned(),
    }
}

#[derive(Debug)]
pub struct StaticFileDownloadError(String);
impl StaticFileDownloadError {
    pub fn new(message: &str) -> StaticFileDownloadError {
        StaticFileDownloadError(message.to_string())
    }
}
impl std::fmt::Display for StaticFileDownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed to download file: {}", self.0)
    }
}
impl Error for StaticFileDownloadError {}

pub async fn with_static_content(main: String, license_name: &str, source: &str, standard: &str,
                                 files_to_process: Vec<OutputFile>) -> Result<Vec<MergedFile>, impl Error> {
    let mut downloaded_files = Vec::new();
    let files_to_process_not_empty = if files_to_process.is_empty() {
        get_all_files()
    } else {
        files_to_process
    };
    for file in files_to_process_not_empty {
        let file_name = file.to_string();
        let content = if file == OutputFile::Main {
            main.clone()
        } else {
            match download_static_content(source, standard, &file_name).await {
                content if !content.is_empty() && file == OutputFile::Cargo =>
                    add_author_and_license(content, Some(license_name.to_owned())),
                content if !content.is_empty() => doc_comments_remove(&content),
                _ => {
                    return Err(StaticFileDownloadError::new(&format!("Static content {} could not be downloaded.", file_name)));
                }
            }
        };
        downloaded_files.push(MergedFile { name: file_name, content });
    }

    Ok(downloaded_files)
}
