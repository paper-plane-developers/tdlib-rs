// Copyright 2021 - developers of the `tdgrand` project.
// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Code to generate Rust's `fn`'s from TL definitions.

use crate::grouper;
use crate::metadata::Metadata;
use crate::rustifier;
use tdgrand_tl_parser::tl::{Category, Definition, ParameterType};
use std::io::{self, Write};

/// Get the list of generic parameters:
///
/// ```ignore
/// <X, Y>
/// ```
fn get_generic_param_list(def: &Definition, declaring: bool) -> String {
    let mut result = String::new();
    for param in def.params.iter() {
        match param.ty {
            ParameterType::Flags => {}
            ParameterType::Normal { ref ty, .. } => {
                if ty.generic_ref {
                    if result.is_empty() {
                        result.push('<');
                    } else {
                        result.push_str(", ");
                    }
                    result.push_str(&ty.name);
                    if declaring {
                        result.push_str(": crate::RemoteCall");
                    }
                }
            }
        }
    }
    if !result.is_empty() {
        result.push('>');
    }
    result
}

/// Defines the `fn` corresponding to the definition:
///
/// ```ignore
/// pub async fn Name(field: Type) {
///     // Content
/// }
/// ```
fn write_fn<W: Write>(
    file: &mut W,
    indent: &str,
    def: &Definition,
    _metadata: &Metadata,
) -> io::Result<()> {
    // Define function
    writeln!(file, "{}", rustifier::definitions::description(def, indent))?;
    write!(
        file,
        "{}pub async fn {}{}(client_id: i32",
        indent,
        rustifier::definitions::function_name(def),
        get_generic_param_list(def, true),
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

/// Writes an entire definition as Rust code (`fn`).
fn write_definition<W: Write>(
    file: &mut W,
    indent: &str,
    def: &Definition,
    metadata: &Metadata,
) -> io::Result<()> {
    write_fn(file, indent, def, metadata)?;
    Ok(())
}

/// Write the entire module dedicated to functions.
pub(crate) fn write_functions_mod<W: Write>(
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
            write_definition(&mut file, indent, definition, metadata)?;
        }

        // End possibly inner mod
        if !key.is_empty() {
            writeln!(file, "    }}")?;
        }
    }

    // End outermost mod
    writeln!(file, "}}")
}
