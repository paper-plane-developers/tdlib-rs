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

/// Defines the `struct` corresponding to the definition:
///
/// ```ignore
/// pub struct Name {
///     pub field: Option<Type>,
/// }
/// ```
fn write_struct<W: Write>(file: &mut W, def: &Definition, _metadata: &Metadata) -> io::Result<()> {
    // Define struct
    writeln!(file, "{}", rustifier::definitions::description(def, "    "))?;
    writeln!(file, "    #[derive(Default, Serialize)]")?;
    writeln!(
        file,
        "    pub struct {} {{",
        rustifier::definitions::type_name(def),
    )?;

    for param in def.params.iter() {
        writeln!(
            file,
            "        {}: Option<{}>,",
            rustifier::parameters::attr_name(param),
            rustifier::parameters::qual_name(param),
        )?;
    }
    writeln!(file, "    }}")?;
    Ok(())
}

/// Defines the `impl` corresponding to the definition:
///
/// ```ignore
/// impl Type {
/// }
/// ```
fn write_impl<W: Write>(file: &mut W, def: &Definition, _metadata: &Metadata) -> io::Result<()> {
    writeln!(
        file,
        "    impl {} {{",
        rustifier::definitions::type_name(def),
    )?;

    writeln!(
        file,
        "        pub fn new() -> {} {{",
        rustifier::definitions::type_name(def)
    )?;
    writeln!(file, "            Default::default()")?;
    writeln!(file, "        }}")?;

    for param in def.params.iter() {
        writeln!(
            file,
            "{}",
            rustifier::parameters::description(param, "        ")
        )?;
        writeln!(
            file,
            "        pub fn {0}(mut self, {0}: {1}) -> {2} {{",
            rustifier::parameters::attr_name(param),
            rustifier::parameters::qual_name(param),
            rustifier::definitions::type_name(def),
        )?;
        writeln!(
            file,
            "            self.{0} = Some({0});",
            rustifier::parameters::attr_name(param)
        )?;
        writeln!(file, "            self")?;
        writeln!(file, "        }}")?;
    }

    writeln!(
        file,
        "        pub async fn send(self, client_id: i32) -> Result<{}, crate::types::Error> {{",
        rustifier::types::qual_name(&def.ty),
    )?;
    writeln!(
        file,
        "            let mut request = serde_json::to_value(self).unwrap();",
    )?;
    writeln!(
        file,
        "            request[\"@type\"] = serde_json::to_value(\"{}\").unwrap();",
        def.name
    )?;
    writeln!(
        file,
        "            let response = send_request(client_id, request).await;",
    )?;
    writeln!(file, "            if response[\"@type\"] == \"error\" {{",)?;
    writeln!(
        file,
        "                return Err(serde_json::from_value(response).unwrap())",
    )?;
    writeln!(file, "            }}")?;
    writeln!(
        file,
        "            Ok(serde_json::from_value(response).unwrap())",
    )?;
    writeln!(file, "        }}")?;

    writeln!(file, "    }}")?;
    Ok(())
}

/// Writes an entire definition as Rust code (`fn`).
fn write_definition<W: Write>(
    file: &mut W,
    def: &Definition,
    metadata: &Metadata,
) -> io::Result<()> {
    write_struct(file, def, metadata)?;
    write_impl(file, def, metadata)?;
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

    let functions = definitions
        .iter()
        .filter(|d| d.category == Category::Functions);

    for definition in functions {
        write_definition(&mut file, definition, metadata)?;
    }

    // End outermost mod
    writeln!(file, "}}")
}
