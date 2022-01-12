// Copyright 2020 - developers of the `grammers` project.
// Copyright 2021 - developers of the `tdgrand` project.
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
use std::io::{self, Write};
use tdgrand_tl_parser::tl::{Category, Definition};

/// Get the list of generic parameters:
///
/// ```ignore
/// <X, Y>
/// ```
fn get_generic_param_list(def: &Definition, declaring: bool) -> String {
    let mut result = String::new();
    for param in def.params.iter() {
        if param.ty.generic_ref {
            if result.is_empty() {
                result.push('<');
            } else {
                result.push_str(", ");
            }
            result.push_str(&param.ty.name);
            if declaring {
                result.push_str(": crate::RemoteCall");
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
///     pub field: Option<Type>,
/// }
/// ```
fn write_struct<W: Write>(
    file: &mut W,
    indent: &str,
    def: &Definition,
    _metadata: &Metadata,
) -> io::Result<()> {
    // Define struct
    writeln!(file, "{}", rustifier::definitions::description(def, indent))?;
    writeln!(file, "{}#[derive(Default, Serialize)]", indent)?;
    writeln!(
        file,
        "{}pub struct {}{} {{",
        indent,
        rustifier::definitions::type_name(def),
        get_generic_param_list(def, true),
    )?;

    for param in def.params.iter() {
        writeln!(
            file,
            "{}    {}: Option<{}>,",
            indent,
            rustifier::parameters::attr_name(param),
            rustifier::parameters::qual_name(param),
        )?;
    }
    writeln!(file, "{}}}", indent)?;
    Ok(())
}

/// Defines the `impl` corresponding to the definition:
///
/// ```ignore
/// impl Type {
/// }
/// ```
fn write_impl<W: Write>(
    file: &mut W,
    indent: &str,
    def: &Definition,
    _metadata: &Metadata,
) -> io::Result<()> {
    writeln!(
        file,
        "{}impl {} {{",
        indent,
        rustifier::definitions::type_name(def),
    )?;

    writeln!(
        file,
        "{}    pub fn new() -> {} {{",
        indent,
        rustifier::definitions::type_name(def)
    )?;
    writeln!(file, "{}        Default::default()", indent)?;
    writeln!(file, "{}    }}", indent)?;

    for param in def.params.iter() {
        writeln!(
            file,
            "{}",
            rustifier::parameters::description(param, &format!("{}    ", indent))
        )?;
        writeln!(
            file,
            "{}    pub fn {1}(mut self, {1}: {2}) -> {3} {{",
            indent,
            rustifier::parameters::attr_name(param),
            rustifier::parameters::qual_name(param),
            rustifier::definitions::type_name(def),
        )?;
        writeln!(
            file,
            "{}        self.{1} = Some({1});",
            indent,
            rustifier::parameters::attr_name(param)
        )?;
        writeln!(file, "{}        self", indent)?;
        writeln!(file, "{}    }}", indent)?;
    }

    writeln!(
        file,
        "{}    pub async fn send(self, client_id: i32) -> Result<{}, crate::types::Error> {{",
        indent,
        rustifier::types::qual_name(&def.ty),
    )?;
    writeln!(
        file,
        "{}        let mut request = serde_json::to_value(self).unwrap();",
        indent
    )?;
    writeln!(
        file,
        "{}        request[\"@type\"] = serde_json::to_value(\"{}\").unwrap();",
        indent, def.name
    )?;
    writeln!(
        file,
        "{}        let response = send_request(client_id, request).await;",
        indent
    )?;
    writeln!(
        file,
        "{}        if response[\"@type\"] == \"error\" {{",
        indent
    )?;
    writeln!(
        file,
        "{}            return Err(serde_json::from_value(response).unwrap())",
        indent
    )?;
    writeln!(file, "{}        }}", indent)?;
    writeln!(
        file,
        "{}        Ok(serde_json::from_value(response).unwrap())",
        indent
    )?;
    writeln!(file, "{}    }}", indent)?;

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
    write_struct(file, indent, def, metadata)?;
    write_impl(file, indent, def, metadata)?;
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
    writeln!(file, "    use serde::Serialize;")?;
    writeln!(file, "    use crate::send_request;")?;

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
