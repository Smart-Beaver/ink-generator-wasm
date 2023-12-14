use serde::{Deserialize, Serialize};
use web_sys::Storage;

use crate::logger::console_log;
use crate::logger::log;

const CACHE_KEY_PREFIX: &str = "sb_wasm_cache_";

//TODO use some normal timestamp type
pub type Timestamp = f64;

pub const EXPIRATION_TIME_MILISECONDS: f64 = 10.0 * 60.0 * 1000.0;//10 min

#[derive(Serialize, Deserialize, Debug)]
pub struct CacheWrapper {
    file_content: String,
    expiration_date: Option<Timestamp>,
}

fn storage_ref() -> Option<Storage> {
    //Window object should always be available
    match web_sys::window().unwrap().local_storage() {
        Ok(local_storage_opt) => local_storage_opt,
        Err(_) => {
            console_log!("Failed to get local storage");
            None
        }
    }
}

pub fn cache_file(filepath: &str, content: &str, expiration_date: Option<Timestamp>) {
    if let Some(storage) = storage_ref() {
        let cache_result = serde_json::to_string(&CacheWrapper {
            file_content: content.to_string(),
            expiration_date,
        }).map(|serialized_value| {
            storage.set_item(format!("{}{}", CACHE_KEY_PREFIX, filepath).as_str(), serialized_value.as_str())
        });

        match cache_result {
            Ok(Ok(content)) => {
                content
            }
            _ => {
                console_log!("Failed to cache file: {}", filepath)
            }
        }
    }
}


pub fn try_load_from_cache(filepath: &str) -> Option<String> {
    if let Some(storage) = storage_ref() {
        return storage.get_item(format!("{}{}", CACHE_KEY_PREFIX, filepath).as_str()).unwrap_or_else(|e| {
            console_log!("Failed to load: {} from cache [Js error: {:?}]", filepath, e);
            None
        }).map(|c| {
            //deserialize
            serde_json::from_str::<CacheWrapper>(c.as_str()).ok()
        }).filter(|c| {
            //check expiration date
            match c {
                None => {
                    console_log!("Failed to deserialize: {}", filepath);
                    false
                }
                Some(c) => {
                    match c.expiration_date {
                        None => true,
                        Some(time) => {
                            time > js_sys::Date::now()
                        }
                    }
                }
            }
        }).map(|c| {
            //get content
            c.unwrap().file_content
        });
    }
    None
}