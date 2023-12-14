use std::error::Error;
use std::ops::Add;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::code_loader::cache_proxy::{cache_file, EXPIRATION_TIME_MILISECONDS, try_load_from_cache};
use crate::logger::console_log;
use crate::logger::log;

#[derive(Debug)]
pub struct DownloadError(String, u16);

impl DownloadError {
    pub fn new(message: &str, status_code: u16) -> DownloadError {
        DownloadError(message.to_string(), status_code)
    }
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed to download file: {}, status code: {}", self.0, self.1)
    }
}

impl Error for DownloadError {}

pub async fn load_source(filepath: &str) -> Result<String, impl Error> {
    match try_load_from_cache(filepath) {
        None => {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);
            let window = web_sys::window().unwrap();
            let request = Request::new_with_str_and_init(filepath, &opts).unwrap();
            let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
            let resp: Response = resp_value.dyn_into().unwrap();
            if resp.status() >= 300 {
                return Err(DownloadError::new(filepath, resp.status()));
            }
            let response_content = JsFuture::from(resp.text().unwrap()).await.unwrap().as_string().unwrap();
            console_log!("Downloaded: {}", filepath);
            cache_file(filepath, &response_content, Some(js_sys::Date::now().add(EXPIRATION_TIME_MILISECONDS)));
            Ok(response_content)
        }
        Some(cached_content) => {
            console_log!("Loaded from cache: {}", filepath);
            Ok(cached_content)
        }
    }
}
