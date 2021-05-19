// Copyright 2021 - developers of the `tdgrand` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;
use tokio::sync::oneshot;

pub(super) struct Observer {
    requests: RwLock<HashMap<String, oneshot::Sender<Value>>>,
}

impl Observer {
    pub fn new() -> Self {
        Observer {
            requests: RwLock::default(),
        }
    }

    pub fn subscribe(&self, extra: String) -> oneshot::Receiver<Value> {
        let (sender, receiver) = oneshot::channel();
        self.requests.write().unwrap().insert(extra, sender);
        receiver
    }

    pub fn notify(&self, response: Value) {
        if let Some(extra) = response["@extra"].as_str() {
            match self.requests.write().unwrap().remove(extra) {
                Some(sender) => {
                    if sender.send(response).is_err() {
                        log::warn!("Got a response of an unaccessible request");
                    }
                }
                None => {
                    log::warn!("Got a response of an unknown request");
                }
            }
        }
    }
}
