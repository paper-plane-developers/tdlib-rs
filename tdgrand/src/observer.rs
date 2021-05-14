// Copyright 2021 - developers of the `tdgrand` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::collections::HashMap;
use std::sync::RwLock;
use tokio::sync::oneshot;

pub(super) struct Observer {
    requests: RwLock<HashMap<String, oneshot::Sender<bool>>>,
}

impl Observer {
    pub fn new() -> Self {
        Observer {
            requests: RwLock::default(),
        }
    }

    pub fn subscribe(&self, extra: String) -> oneshot::Receiver<bool> {
        let (sender, receiver) = oneshot::channel();
        self.requests.write().unwrap().insert(extra, sender);
        receiver
    }

    pub fn notify(&self, extra: &str) {
        match self.requests.write().unwrap().remove(extra) {
            Some(sender) => {
                if let Err(_) = sender.send(true) {
                    log::warn!("Got a response of an unaccessible request");
                }
            }
            None => {
                log::warn!("Got a response of an unknown request");
            }
        }
    }
}
