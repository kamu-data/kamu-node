use clap::*;
use indoc::indoc;

pub fn cli(binary_name: &'static str, version: &'static str) -> App<'static, 'static> {
    App::new(binary_name)
        .global_settings(&[AppSettings::ColoredHelp])
        .settings(&[AppSettings::SubcommandRequiredElseHelp])
        .version(version)
        .after_help(indoc!(
            r"
            To get help around individual commands use:
              kamu-api-server <command> -h
              kamu-api-server <command> <sub-command> -h
            "
        ))
        .args(&[Arg::with_name("metadata-repo")
            .long("metadata-repo")
            .takes_value(true)
            .validator(|arg| {
                let url = url::Url::parse(&arg).map_err(|e| e.to_string())?;
                if url.scheme() == "file" {
                    url.to_file_path()
                        .map_err(|_| "Invalid URL, should be in form: file:///home/me/workspace")?;
                }
                Ok(())
            })
            .help("URL of the dataset metadata repository")])
        .subcommands(vec![
            SubCommand::with_name("run").about("Run the server").args(&[
                Arg::with_name("address")
                    .long("address")
                    .takes_value(true)
                    .default_value("127.0.0.1")
                    .help("Address of the interface to bind to"),
                Arg::with_name("http-port")
                    .long("http-port")
                    .takes_value(true)
                    .default_value("8080")
                    .help("Port to listen for HTTP traffic on"),
            ]),
            SubCommand::with_name("gql")
                .about("GraphQL related command group")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(SubCommand::with_name("schema").about("Prints out GraphQL schema"))
                .subcommand(
                    SubCommand::with_name("query")
                        .about("Executes the GraphQL query and prints out the result")
                        .args(&[
                            Arg::with_name("full")
                                .long("full")
                                .help("Display the full result including extensions"),
                            Arg::with_name("query").index(1).required(true),
                        ])
                        .after_help(indoc!(
                            r"
                            Example:
                                kamu-api-server gql query '{ apiVersion }'
                            "
                        )),
                ),
        ])
}
