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

fn write_function_declaration<W: Write>(
    file: &mut W,
    indent: &str,
    def: &Definition,
    _metadata: &Metadata,
) -> io::Result<()> {
    // Define function
    write!(
        file,
        "{}    fn {}(&self, client_id: i32",
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
    writeln!(file, ");")?;
    Ok(())
}

fn write_function_definition<W: Write>(
    file: &mut W,
    indent: &str,
    def: &Definition,
    _metadata: &Metadata,
) -> io::Result<()> {
    // Define function
    write!(
        file,
        "{}    fn {}(&self, client_id: i32",
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
    writeln!(file, ") {{")?;

    // Write function content
    writeln!(file, "{}        let json = serde_json::json!({{", indent)?;
    writeln!(file, "{}            \"@type\": \"{}\",", indent, def.name)?;
    for param in def.params.iter() {
        match param.ty {
            ParameterType::Flags => {
                // Flags are computed on-the-fly, not stored
            }
            ParameterType::Normal { .. } => {
                writeln!(
                    file,
                    "{}            \"{1}\": {1},",
                    indent,
                    rustifier::parameters::attr_name(param),
                )?;
            }
        }
    }
    writeln!(file, "{}        }});", indent)?;
    writeln!(file, "{}        self.send(client_id, &json.to_string());", indent)?;

    writeln!(file, "{}    }}", indent)?;
    Ok(())
}

/// Write the entire module dedicated to functions.
pub(crate) fn write_functions_mod<W: Write>(
    mut file: &mut W,
    definitions: &[Definition],
    metadata: &Metadata,
) -> io::Result<()> {
    // Begin outermost mod
    write!(
        file,
        "\
         pub mod functions {{\n\
         "
    )?;

    let grouped = grouper::group_by_ns(definitions, Category::Functions);
    let mut sorted_keys: Vec<&String> = grouped.keys().collect();
    sorted_keys.sort();
    for key in sorted_keys.into_iter() {
        // Begin possibly inner mod
        let indent = if key.is_empty() {
            "    "
        } else {
            writeln!(file, "    #[allow(clippy::unreadable_literal)]")?;
            writeln!(file, "    pub mod {} {{", key)?;
            "        "
        };

        // Begin ClientManagerExt trait
        writeln!(file, "{}pub trait ClientManagerExt {{", indent)?;

        for definition in grouped[key]
            .iter()
            .filter(|def| def.category == Category::Functions || !ignore_type(&def.ty))
        {
            write_function_declaration(&mut file, indent, definition, metadata)?;
        }

        // End ClientManagerExt trait
        writeln!(file, "{}}}", indent)?;

        // Begin ClientManagerExt impl
        writeln!(file, "{}impl ClientManagerExt for crate::ClientManager {{", indent)?;

        for definition in grouped[key]
            .iter()
            .filter(|def| def.category == Category::Functions || !ignore_type(&def.ty))
        {
            write_function_definition(&mut file, indent, definition, metadata)?;
        }

        // End ClientManagerExt impl
        writeln!(file, "{}}}", indent)?;

        // End possibly inner mod
        if !key.is_empty() {
            writeln!(file, "        }}")?;
        }
    }

    // End outermost mod
    writeln!(file, "}}")
}
