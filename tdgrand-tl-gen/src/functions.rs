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
use crate::metadata::Metadata;
use crate::rustifier;
use grammers_tl_parser::tl::{Category, Definition, ParameterType};
use std::io::{self, Write};

fn write_function<W: Write>(
    file: &mut W,
    indent: &str,
    def: &Definition,
    _metadata: &Metadata,
) -> io::Result<()> {
    // Define function
    write!(
        file,
        "{}pub async fn {}(client_id: i32",
        indent,
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
    writeln!(file, ") -> Result<{}, crate::types::Error> {{", rustifier::types::qual_name(&def.ty))?;

    // Write function content
    writeln!(file, "{}    let request = json!({{", indent)?;
    writeln!(file, "{}        \"@type\": \"{}\",", indent, def.name)?;
    for param in def.params.iter() {
        match param.ty {
            ParameterType::Flags => {
                // Flags are computed on-the-fly, not stored
            }
            ParameterType::Normal { .. } => {
                writeln!(
                    file,
                    "{}        \"{1}\": {1},",
                    indent,
                    rustifier::parameters::attr_name(param),
                )?;
            }
        }
    }
    writeln!(file, "{}    }});", indent)?;
    writeln!(file, "{}    let response = send_request(client_id, request).await;", indent)?;
    writeln!(file, "{}    if response[\"@type\"] == \"error\" {{", indent)?;
    writeln!(file, "{}        return Err(serde_json::from_value(response).unwrap())", indent)?;
    writeln!(file, "{}    }}", indent)?;
    writeln!(file, "{}    Ok(serde_json::from_value(response).unwrap())", indent)?;

    writeln!(file, "{}}}", indent)?;
    Ok(())
}

/// Write the entire module dedicated to functions.
pub(crate) fn write_client_mod<W: Write>(
    mut file: &mut W,
    definitions: &[Definition],
    metadata: &Metadata,
) -> io::Result<()> {
    // Begin outermost mod
    writeln!(file, "pub mod functions {{")?;
    writeln!(file, "    use crate::send_request;")?;
    writeln!(file, "    use serde_json::json;")?;

    let grouped = grouper::group_by_ns(definitions, Category::Functions);
    let mut sorted_keys: Vec<&String> = grouped.keys().collect();
    sorted_keys.sort();
    for key in sorted_keys.into_iter() {
        // Begin possibly inner mod
        let indent = if key.is_empty() {
            "    "
        } else {
            writeln!(file, "    pub mod {} {{", key)?;
            "        "
        };

        for definition in grouped[key].iter() {
            write_function(&mut file, indent, definition, metadata)?;
        }

        // End possibly inner mod
        if !key.is_empty() {
            writeln!(file, "    }}")?;
        }
    }

    // End outermost mod
    writeln!(file, "}}")
}
