// Copyright 2020 - developers of the `grammers` project.
// Copyright 2021 - developers of the `tdgrand` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Code to generate Rust's `struct`'s from TL definitions.

use crate::grouper;
use crate::ignore_type;
use crate::metadata::Metadata;
use crate::rustifier;
use std::io::{self, Write};
use tdgrand_tl_parser::tl::{Category, Definition};

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
    writeln!(file, "{}", rustifier::definitions::description(def, indent))?;
    writeln!(
        file,
        "{}#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]",
        indent
    )?;
    writeln!(
        file,
        "{}pub struct {} {{",
        indent,
        rustifier::definitions::type_name(def),
    )?;

    for param in def.params.iter() {
        writeln!(
            file,
            "{}",
            rustifier::parameters::description(param, &format!("{}    ", indent))
        )?;
        if let Some(serde_with) = rustifier::parameters::serde_with(param) {
            writeln!(file, "{}    #[serde(with = \"{}\")]", indent, serde_with)?;
        }
        write!(
            file,
            "{}    pub {}: ",
            indent,
            rustifier::parameters::attr_name(param),
        )?;

        let is_optional = param.description.contains("may be null");
        if is_optional {
            write!(file, "Option<")?;
        }
        write!(file, "{}", rustifier::parameters::qual_name(param))?;
        if is_optional {
            write!(file, ">")?;
        }
        writeln!(file, ",")?;
    }
    writeln!(file, "{}}}", indent)?;
    Ok(())
}

/// Writes an entire definition as Rust code (`struct`).
fn write_definition<W: Write>(
    file: &mut W,
    indent: &str,
    def: &Definition,
    metadata: &Metadata,
) -> io::Result<()> {
    write_struct(file, indent, def, metadata)?;
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
