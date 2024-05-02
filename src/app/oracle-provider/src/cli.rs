// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help(true))]
pub struct Cli {
    /// Config file path
    #[arg(long, default_value = "config.yaml")]
    pub config: PathBuf,

    #[command(subcommand)]
    run: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Run(RunArgs),
}

#[derive(Debug, clap::Args)]
pub struct RunArgs {}
