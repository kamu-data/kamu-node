use clap::*;
use indoc::indoc;

pub fn cli(binary_name: &'static str, version: &'static str) -> App<'static> {
    App::new(binary_name)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(version)
        .after_help(indoc!(
            r"
            To get help around individual commands use:
              kamu-api-server <command> -h
              kamu-api-server <command> <sub-command> -h
            "
        ))
        .args(&[Arg::new("metadata-repo")
            .long("metadata-repo")
            .takes_value(true)
            .validator(|arg: &str| -> std::result::Result<(), String> {
                let url = url::Url::parse(arg).map_err(|e| e.to_string())?;
                if url.scheme() == "file" {
                    url.to_file_path()
                        .map_err(|_| "Invalid URL, should be in form: file:///home/me/workspace")?;
                }
                Ok(())
            })
            .help("URL of the dataset metadata repository")])
        .subcommands(vec![
            App::new("run").about("Run the server").args(&[
                Arg::new("address")
                    .long("address")
                    .takes_value(true)
                    .default_value("127.0.0.1")
                    .help("Address of the interface to bind to"),
                Arg::new("http-port")
                    .long("http-port")
                    .takes_value(true)
                    .default_value("8080")
                    .help("Port to listen for HTTP traffic on"),
            ]),
            App::new("gql")
                .about("GraphQL related command group")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(App::new("schema").about("Prints out GraphQL schema"))
                .subcommand(
                    App::new("query")
                        .about("Executes the GraphQL query and prints out the result")
                        .args(&[
                            Arg::new("full")
                                .long("full")
                                .help("Display the full result including extensions"),
                            Arg::new("query").index(1).required(true),
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
