// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Preparing the build information
    let build = vergen_gitcl::BuildBuilder::all_build()?;
    let cargo = vergen_gitcl::CargoBuilder::all_cargo()?;
    let gitcl = vergen_gitcl::GitclBuilder::all_git()?;
    let rustc = vergen_gitcl::RustcBuilder::all_rustc()?;

    vergen_gitcl::Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&gitcl)?
        .add_instructions(&rustc)?
        .emit()?;

    Ok(())
}
