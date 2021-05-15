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
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RawVec<T>(pub Vec<T>);

pub fn next_update() -> Option<Update> {
    let response = tdjson::receive(2.0);
    if let Some(response) = response {
        println!("{}", response);

        let json: Value = serde_json::from_str(&response).unwrap();

        if let Some(td_extra) = json["@extra"].as_str() {
            OBSERVER.notify(td_extra);
        }

        let td_type = json["@type"].as_str().unwrap();
        if td_type.starts_with("update") {
            return Some(serde_json::from_value(json).unwrap());
        }
    }

    None
}
