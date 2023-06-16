// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::PathBuf;

use clap::*;
use indoc::indoc;

pub fn cli(binary_name: &'static str, version: &'static str) -> Command {
    Command::new(binary_name)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .version(version)
        .after_help(indoc!(
            r"
            To get help around individual commands use:
              kamu-api-server <command> -h
              kamu-api-server <command> <sub-command> -h
            "
        ))
        .args([
            Arg::new("repo-url")
                .long("repo-url")
                .value_parser(value_parse_metadata_repo)
                .help("URL of the remote dataset repository"),
            Arg::new("local-repo")
                .long("local-repo")
                .value_parser(value_parser!(PathBuf))
                .help("Path to the local dataset repository"),
        ])
        .subcommands([
            Command::new("run").about("Run the server").args([
                Arg::new("address")
                    .long("address")
                    .value_parser(value_parser!(std::net::IpAddr))
                    .help("Expose HTTP server on specific network interface"),
                Arg::new("http-port")
                    .long("http-port")
                    .value_parser(value_parser!(u16))
                    .help("Expose HTTP server on specific port"),
            ]),
            Command::new("gql")
                .about("GraphQL related command group")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommands([
                    Command::new("schema").about("Prints out GraphQL schema"),
                    Command::new("query")
                        .about("Executes the GraphQL query and prints out the result")
                        .args(&[
                            Arg::new("full")
                                .long("full")
                                .action(ArgAction::SetTrue)
                                .help("Display the full result including extensions"),
                            Arg::new("query").index(1).required(true),
                        ])
                        .after_help(indoc!(
                            r"
                            Example:
                                kamu-api-server gql query '{ apiVersion }'
                            "
                        )),
                ]),
        ])
}

fn value_parse_metadata_repo(s: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(s).map_err(|e| e.to_string())?;
    if url.scheme() == "file" {
        url.to_file_path()
            .map_err(|_| "Invalid URL, should be in form: file:///home/me/workspace")?;
    }
    Ok(url)
}
