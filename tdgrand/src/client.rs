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
use std::os::raw::{c_char, c_double, c_void};

type TDLibClient = *mut c_void;

pub struct Client {
    instance: TDLibClient,
}

#[link(name = "tdjson")]
extern "C" {
    fn td_json_client_create() -> TDLibClient;
    fn td_json_client_send(client: TDLibClient, request: *const c_char);
    fn td_json_client_receive(client: TDLibClient, timeout: c_double) -> *mut c_char;
    fn td_json_client_destroy(client: TDLibClient);
}

impl Client {
    pub fn new() -> Self {
        let client = unsafe { td_json_client_create() };
        Client { instance: client }
    }

    pub fn next_update(&self) -> Option<Update> {
        let response = self.receive(10.0);

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

    pub(crate) fn send(&self, request: &str) {
        let cstring = CString::new(request).unwrap();
        unsafe { td_json_client_send(self.instance, cstring.as_ptr()) }
    }

    fn receive(&self, timeout: f64) -> Option<String> {
        unsafe {
            match td_json_client_receive(self.instance, timeout)
                .as_ref()
                .map(|response| CStr::from_ptr(response).to_string_lossy().into_owned())
            {
                None => {
                    None
                }
                Some(contents) => {
                    Some(contents)
                }
            }
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe { td_json_client_destroy(self.instance) }
    }
}
