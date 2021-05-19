// Copyright 2021 - developers of the `tdgrand` project.
// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Code to generate Rust's `struct`'s from TL definitions.

use crate::grouper;
use crate::metadata::Metadata;
use crate::rustifier;
use crate::ignore_type;
use grammers_tl_parser::tl::{Category, Definition, ParameterType};
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

/// Defines the `struct` corresponding to the definition:
///
/// ```ignore
/// pub struct Name {
///     pub field: Type,
/// }
/// ```
fn write_struct<W: Write>(
    file: &mut W,
    indent: &str,
    def: &Definition,
    _metadata: &Metadata,
) -> io::Result<()> {
    // Define struct
    writeln!(file, "{}#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]", indent)?;
    write!(
        file,
        "{}pub struct {}{} {{",
        indent,
        rustifier::definitions::type_name(def),
        get_generic_param_list(def, true),
    )?;

    writeln!(file, "")?;
    for param in def.params.iter() {
        match param.ty {
            ParameterType::Flags => {
                // Flags are computed on-the-fly, not stored
            }
            ParameterType::Normal { .. } => {
                writeln!(
                    file,
                    "{}    pub {}: {},",
                    indent,
                    rustifier::parameters::attr_name(param),
                    rustifier::parameters::qual_name(param),
                )?;
            }
        }
    }
    writeln!(file, "{}}}", indent)?;
    Ok(())
}

/// Write the entire module dedicated to types.
pub(crate) fn write_types_mod<W: Write>(
    mut file: &mut W,
    definitions: &[Definition],
    metadata: &Metadata,
) -> io::Result<()> {
    // Begin outermost mod
    writeln!(file, "pub mod types {{")?;
    writeln!(file, "    use serde::{{Deserialize, Serialize}};")?;

    let grouped = grouper::group_by_ns(definitions, Category::Types);
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

        for definition in grouped[key]
            .iter()
            .filter(|def| !def.params.is_empty() && !ignore_type(&def.ty))
        {
            write_struct(&mut file, indent, definition, metadata)?;
        }

        // End possibly inner mod
        if !key.is_empty() {
            writeln!(file, "    }}")?;
        }
    }

    // End outermost mod
    writeln!(file, "}}")
}
