// Copyright 2021 - developers of the `tdgrand` project.
// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Code to generate Rust's `enum`'s from TL definitions.

use crate::grouper;
use crate::metadata::Metadata;
use crate::rustifier;
use crate::ignore_type;
use grammers_tl_parser::tl::{Definition, Type};
use std::io::{self, Write};

/// Writes an enumeration listing all types such as the following rust code:
///
/// ```ignore
/// pub enum Name {
///     Variant(crate::types::Name),
/// }
/// ```
fn write_enum<W: Write>(
    file: &mut W,
    indent: &str,
    ty: &Type,
    metadata: &Metadata,
) -> io::Result<()> {

    writeln!(file, "{}#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]", indent)?;
    writeln!(file, "{}#[serde(tag = \"@type\")]", indent)?;
    writeln!(
        file,
        "{}pub enum {} {{",
        indent,
        rustifier::types::type_name(ty)
    )?;
    for d in metadata.defs_with_type(ty) {
        writeln!(file, "{}    #[serde(rename(deserialize = \"{}\"))]", indent, d.name)?;
        write!(
            file,
            "{}    {}",
            indent,
            rustifier::definitions::variant_name(d)
        )?;

        // Variant with no struct since it has no data and it only adds noise
        if d.params.is_empty() {
            writeln!(file, ",")?;
            continue;
        } else {
            write!(file, "(")?;
        }

        if metadata.is_recursive_def(d) {
            write!(file, "Box<")?;
        }
        write!(file, "{}", rustifier::definitions::qual_name(&d))?;
        if metadata.is_recursive_def(d) {
            write!(file, ">")?;
        }

        writeln!(file, "),")?;
    }
    writeln!(file, "{}}}", indent)?;
    Ok(())
}

/// Writes an entire definition as Rust code (`enum` and `impl`).
fn write_definition<W: Write>(
    file: &mut W,
    indent: &str,
    ty: &Type,
    metadata: &Metadata,
) -> io::Result<()> {
    write_enum(file, indent, ty, metadata)?;
    Ok(())
}

/// Write the entire module dedicated to enums.
pub(crate) fn write_enums_mod<W: Write>(
    mut file: &mut W,
    definitions: &[Definition],
    metadata: &Metadata,
) -> io::Result<()> {
    // Begin outermost mod
    writeln!(file, "\
pub mod enums {{
    use serde::{{Deserialize, Serialize}};")?;

    let grouped = grouper::group_types_by_ns(definitions);
    let mut sorted_keys: Vec<&Option<String>> = grouped.keys().collect();
    sorted_keys.sort();
    for key in sorted_keys.into_iter() {
        // Begin possibly inner mod
        let indent = if let Some(ns) = key {
            writeln!(file, "    #[allow(clippy::large_enum_variant)]")?;
            writeln!(file, "    pub mod {} {{", ns)?;
            "        "
        } else {
            "    "
        };

        for ty in grouped[key].iter().filter(|ty| !ignore_type(*ty)) {
            write_definition(&mut file, indent, ty, metadata)?;
        }

        // End possibly inner mod
        if key.is_some() {
            writeln!(file, "    }}")?;
        }
    }

    // End outermost mod
    writeln!(file, "}}")
}
