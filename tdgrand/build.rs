// Copyright 2021 - developers of the `tdgrand` project.
// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use grammers_tl_parser::parse_tl_file;
use grammers_tl_parser::tl::Definition;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use tdgrand_tl_gen::{generate_client, generate_rust_code, Config};

/// Load the type language definitions from a certain file.
/// Parse errors will be printed to `stderr`, and only the
/// valid results will be returned.
fn load_tl(file: &str) -> io::Result<Vec<Definition>> {
    let mut file = File::open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(parse_tl_file(&contents)
        .into_iter()
        .filter_map(|d| match d {
            Ok(d) => Some(d),
            Err(e) => {
                eprintln!("TL: parse error: {:?}", e);
                None
            }
        })
        .collect())
}

fn main() -> std::io::Result<()> {
    let definitions = load_tl("tl/api.tl")?;

    let mut file = BufWriter::new(File::create(
        Path::new(&env::var("OUT_DIR").unwrap()).join("generated.rs"),
    )?);

    let mut client_file = BufWriter::new(File::create(
        Path::new(&env::var("OUT_DIR").unwrap()).join("client.rs"),
    )?);

    let config = Config {
        deserializable_functions: cfg!(feature = "deserializable-functions"),
        impl_debug: cfg!(feature = "impl-debug"),
        impl_from_enum: cfg!(feature = "impl-from-enum"),
        impl_from_type: cfg!(feature = "impl-from-type"),
    };

    generate_rust_code(&mut file, &definitions, &config)?;
    file.flush()?;

    generate_client(&mut client_file, &definitions)?;
    client_file.flush()?;

    Ok(())
}
