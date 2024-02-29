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

pub fn cli() -> Command {
    Command::new(crate::BINARY_NAME)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .version(crate::VERSION)
        .after_help(indoc!(
            r"
            To get help around individual commands use:
              kamu-api-server <command> -h
              kamu-api-server <command> <sub-command> -h
            "
        ))
        .args([
            Arg::new("config")
                .long("config")
                .value_parser(value_parser!(std::path::PathBuf))
                .help("Path to the config file"),
            Arg::new("repo-url")
                .long("repo-url")
                .value_parser(value_parse_repo_url)
                .help("URL of the remote dataset repository"),
            // TODO: This is temporary and will be removed soon
            // See: https://github.com/kamu-data/kamu-cli/issues/342
            Arg::new("multi-tenant")
                .long("multi-tenant")
                .action(ArgAction::SetTrue)
                .help("Indicates that target repo is multi-tenant (for file:// only)"),
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
                Arg::new("flightsql-port")
                    .long("flightsql-port")
                    .value_parser(value_parser!(u16))
                    .help("Expose Flight SQL server on specific port"),
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

/// Allows URLs or local paths
fn value_parse_repo_url(s: &str) -> Result<url::Url, String> {
    match url::Url::parse(s) {
        Ok(url) => Ok(url),
        Err(_) => match PathBuf::from(s).canonicalize() {
            Ok(path) => Ok(url::Url::from_file_path(path).unwrap()),
            Err(_) => Err(
                "Invalid repo-url, should be a path or a URL in form: file:///home/me/workspace"
                    .to_string(),
            ),
        },
    }
}
