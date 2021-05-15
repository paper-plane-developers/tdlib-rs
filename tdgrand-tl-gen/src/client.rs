// Copyright 2021 - developers of the `tdgrand` project.
// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Code to generate Rust's `function`'s from TL definitions.

use crate::grouper;
use crate::ignore_type;
use crate::metadata::Metadata;
use crate::rustifier;
use grammers_tl_parser::tl::{Category, Definition, ParameterType};
use std::io::{self, Write};

fn write_method<W: Write>(
    file: &mut W,
    def: &Definition,
    _metadata: &Metadata,
) -> io::Result<()> {
    // Define method
    write!(
        file,
        "        pub async fn {}(&self",
        rustifier::definitions::function_name(def),
    )?;
    for param in def.params.iter() {
        match param.ty {
            ParameterType::Flags => {
                // Flags are computed on-the-fly, not stored
            }
            ParameterType::Normal { .. } => {
                write!(
                    file,
                    ", {}: {}",
                    rustifier::parameters::attr_name(param),
                    rustifier::parameters::qual_name(param),
                )?;
            }
        }
    }
    writeln!(file, ") -> {} {{", rustifier::types::qual_name(&def.ty))?;

    // Write method content
    writeln!(file, "            let request = json!({{")?;
    writeln!(file, "                \"@type\": \"{}\",", def.name)?;
    for param in def.params.iter() {
        match param.ty {
            ParameterType::Flags => {
                // Flags are computed on-the-fly, not stored
            }
            ParameterType::Normal { .. } => {
                writeln!(
                    file,
                    "                \"{0}\": {0},",
                    rustifier::parameters::attr_name(param),
                )?;
            }
        }
    }
    writeln!(file, "            }});")?;
    writeln!(file, "            let response = self.send_request(request).await;")?;
    writeln!(file, "            serde_json::from_value(response).unwrap()")?;

    writeln!(file, "        }}")?;
    Ok(())
}

/// Write the entire module dedicated to the client.
pub(crate) fn write_client_mod<W: Write>(
    mut file: &mut W,
    definitions: &[Definition],
    metadata: &Metadata,
) -> io::Result<()> {
    // Begin outermost mod and impl
    writeln!(
        file,
        "\
pub mod client {{
    use crate::{{tdjson, OBSERVER}};
    use serde_json::{{json, Value}};
    use uuid::Uuid;
    pub struct Client {{
        client_id: i32,
    }}
    impl Client {{
        pub fn new() -> Self {{
            Client {{
                client_id: tdjson::create_client(),
            }}
        }}
        async fn send_request(&self, mut request: Value) -> Value {{
            let extra = Uuid::new_v4().to_string();
            request[\"@extra\"] = serde_json::to_value(extra.clone()).unwrap();

            let receiver = OBSERVER.subscribe(extra);
            tdjson::send(self.client_id, request.to_string());

            receiver.await.unwrap()
        }}\
    "
    )?;

    let grouped = grouper::group_by_ns(definitions, Category::Functions);
    let mut sorted_keys: Vec<&String> = grouped.keys().collect();
    sorted_keys.sort();
    for key in sorted_keys.into_iter() {
        for definition in grouped[key]
            .iter()
            .filter(|def| !ignore_type(&def.ty))
        {
            write_method(&mut file, definition, metadata)?;
        }
    }

    // End outermost mod and impl
    writeln!(file, "    }}")?;
    writeln!(file, "}}")
}
