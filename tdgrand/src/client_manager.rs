// Copyright 2021 - developers of the `tdgrand` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::enums::Update;
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int};

#[link(name = "tdjson")]
extern "C" {
    fn td_create_client_id() -> c_int;
    fn td_send(client_id: c_int, request: *const c_char);
    fn td_receive(timeout: c_double) -> *const c_char;
}

pub struct ClientManager {}

impl ClientManager {
    pub fn new() -> Self {
        ClientManager {}
    }

    pub fn create_client() -> i32 {
        unsafe { td_create_client_id() }
    }

    pub async fn next_update(&self) -> Option<Update> {
        let response = Self::receive(2.0);

        if let Some(response) = response {
            println!("{}", response);

            let json: Value = serde_json::from_str(&response).unwrap();
            let td_type = json["@type"].as_str().unwrap();

            if td_type.starts_with("update") {
                return Some(serde_json::from_value(json).unwrap());
            }
        }

        None
    }

    pub(crate) fn send(&self, client_id: i32, request: &str) {
        let cstring = CString::new(request).unwrap();
        unsafe { td_send(client_id, cstring.as_ptr()) }
    }

    fn receive(timeout: f64) -> Option<String> {
        unsafe {
            match td_receive(timeout)
                .as_ref()
                .map(|response| CStr::from_ptr(response).to_string_lossy().into_owned())
            {
                None => None,
                Some(contents) => Some(contents),
            }
        }
    }
}
