// Copyright 2021 - developers of the `tdgrand` project.
// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module gathers all the code generation submodules and coordinates
//! them, feeding them the right data.
mod client;
mod enums;
mod grouper;
mod metadata;
mod rustifier;
mod structs;

use grammers_tl_parser::tl::{Definition, Type};
use std::io::{self, Write};

/// Don't generate types for definitions of this type,
/// since they are "core" types and treated differently.
const SPECIAL_CASED_TYPES: [&str; 1] = ["Bool"];

fn ignore_type(ty: &Type) -> bool {
    SPECIAL_CASED_TYPES
        .iter()
        .find(|&&x| x == ty.name)
        .is_some()
}

pub fn generate_rust_code(
    file: &mut impl Write,
    definitions: &[Definition],
) -> io::Result<()> {
    write!(
        file,
        "\
         // Copyright 2021 - developers of the `tdgrand` project.\n\
         // Copyright 2020 - developers of the `grammers` project.\n\
         //\n\
         // Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or\n\
         // https://www.apache.org/licenses/LICENSE-2.0> or the MIT license\n\
         // <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your\n\
         // option. This file may not be copied, modified, or distributed\n\
         // except according to those terms.\n\
         "
    )?;

    let metadata = metadata::Metadata::new(&definitions);
    structs::write_category_mod(file, definitions, &metadata)?;
    enums::write_enums_mod(file, definitions, &metadata)?;

    Ok(())
}

pub fn generate_client(
    file: &mut impl Write,
    definitions: &[Definition],
) -> io::Result<()> {
    write!(
        file,
        "\
         // Copyright 2021 - developers of the `tdgrand` project.\n\
         // Copyright 2020 - developers of the `grammers` project.\n\
         //\n\
         // Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or\n\
         // https://www.apache.org/licenses/LICENSE-2.0> or the MIT license\n\
         // <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your\n\
         // option. This file may not be copied, modified, or distributed\n\
         // except according to those terms.\n\
         "
    )?;

    let metadata = metadata::Metadata::new(&definitions);
    client::write_client_mod(file, definitions, &metadata)?;

    Ok(())
}
