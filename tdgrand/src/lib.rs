// Copyright 2021 - developers of the `tdgrand` project.
// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
mod generated;
mod observer;
mod tdjson;

pub use generated::{client::Client, enums, types};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use enums::Update;

pub(crate) static OBSERVER: Lazy<observer::Observer> =
    Lazy::new(|| observer::Observer::new());

/// This struct represents the concrete type of a vector, that is,
/// `vector` as opposed to the type `Vector`. This bare type is less
/// common, so instead of creating a enum for `Vector` wrapping `vector`
/// as Rust's `Vec` (as we would do with auto-generated code),
/// a new-type for `vector` is used instead.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct RawVec<T>(pub Vec<T>);

/// Receive a single response from TdLib. If the response is an update, it
/// returns a tuple with the `Update` and the associated `client_id`.
/// Note that to start receiving updates for a `Client` you need to send
/// at least a request with it first, so maybe you should request something
/// like `client.test_network()` first.
pub fn step() -> Option<(Update, i32)> {
    let response = tdjson::receive(2.0);
    if let Some(response) = response {
        let response: Value = serde_json::from_str(&response).unwrap();

        match response["@extra"].as_str() {
            Some(_) => {
                OBSERVER.notify(response);
            }
            None => {
                let client_id = response["@client_id"].as_i64().unwrap() as i32;
                match serde_json::from_value(response) {
                    Ok(update) => {
                        return Some((update, client_id));
                    }
                    Err(_) => {
                        log::warn!("Got an unknown response");
                    }
                }
            }
        }
    }

    None
}
