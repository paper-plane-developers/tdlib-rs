// Copyright 2020 - developers of the `grammers` project.
// Copyright 2021 - developers of the `tdgrand` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Code to generate Rust's `fn`'s from TL definitions.

use crate::metadata::Metadata;
use crate::rustifier;
use std::io::{self, Write};
use tdgrand_tl_parser::tl::{Category, Definition};

/// Defines the `function` corresponding to the definition:
///
/// ```ignore
/// pub async fn name(client_id: i32, field: Type) -> Result {
///
/// }
/// ```
fn write_function<W: Write>(
    file: &mut W,
    def: &Definition,
    _metadata: &Metadata,
) -> io::Result<()> {
    writeln!(file, "{}", rustifier::definitions::description(def, "    "))?;

    writeln!(file, "    /// # Arguments")?;

    for param in def.params.iter() {
        writeln!(
            file,
            "    /// * `{}` - {}",
            rustifier::parameters::attr_name(param),
            param.description.replace('\n', "\n    /// ")
        )?;
    }
    writeln!(
        file,
        "    /// * `client_id` - The client id to send the request to"
    )?;

    write!(
        file,
        "    pub async fn {}(",
        rustifier::definitions::function_name(def)
    )?;

    for param in def.params.iter() {
        write!(file, "{}: ", rustifier::parameters::attr_name(param))?;

        let is_optional = rustifier::parameters::is_optional(param);
        if is_optional {
            write!(file, "Option<")?;
        }

        write!(file, "{}", rustifier::parameters::qual_name(param))?;

        if is_optional {
            write!(file, ">")?;
        }

        write!(file, ", ")?;
    }

    writeln!(
        file,
        "client_id: i32) -> Result<{}, crate::types::Error> {{",
        rustifier::types::qual_name(&def.ty)
    )?;

    writeln!(file, "        let request = json!({{")?;
    writeln!(file, "            \"@type\": \"{}\",", def.name)?;
    for param in def.params.iter() {
        writeln!(
            file,
            "            \"{0}\": {0},",
            rustifier::parameters::attr_name(param),
        )?;
    }
    writeln!(file, "        }});")?;

    writeln!(
        file,
        "        let response = send_request(client_id, request).await;"
    )?;
    writeln!(file, "        if response[\"@type\"] == \"error\" {{")?;
    writeln!(
        file,
        "            return Err(serde_json::from_value(response).unwrap())"
    )?;
    writeln!(file, "        }}")?;
    writeln!(
        file,
        "        Ok(serde_json::from_value(response).unwrap())"
    )?;

    writeln!(file, "    }}")?;
    Ok(())
}

/// Writes an entire definition as Rust code (`fn`).
fn write_definition<W: Write>(
    file: &mut W,
    def: &Definition,
    metadata: &Metadata,
) -> io::Result<()> {
    write_function(file, def, metadata)?;
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
    writeln!(file, "    use serde_json::json;")?;
    writeln!(file, "    use crate::send_request;")?;

    let functions = definitions
        .iter()
        .filter(|d| d.category == Category::Functions);

    for definition in functions {
        write_definition(&mut file, definition, metadata)?;
    }

    // End outermost mod
    writeln!(file, "}}")
}
