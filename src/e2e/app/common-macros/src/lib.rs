// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_str, Expr, Ident, LitStr, Path, Token};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro]
pub fn kamu_node_run_api_server_e2e_test_with_repo(input: TokenStream) -> TokenStream {
    let InputArgs {
        storage,
        fixture,
        options,
        extra_test_groups,
        ..
    } = parse_macro_input!(input as InputArgs);

    let options = options.unwrap_or_else(|| syn::parse_str("Options::default()").unwrap());
    let extra_test_groups =
        extra_test_groups.unwrap_or_else(|| syn::LitStr::new("", proc_macro2::Span::call_site()));

    let expanded = quote! {
        kamu_node_run_api_server_e2e_test!(
            storage = #storage,
            fixture = #fixture,
            repo_type = local_fs,
            options = #options,
            extra_test_groups = #extra_test_groups,
        );

        kamu_node_run_api_server_e2e_test!(
            storage = #storage,
            fixture = #fixture,
            repo_type = s3,
            options = #options,
            extra_test_groups = #extra_test_groups,
        );
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn kamu_node_run_api_server_e2e_test(input: TokenStream) -> TokenStream {
    let harness_method = parse_str("run_api_server").unwrap();

    kamu_node_e2e_test_impl(&harness_method, input)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro]
pub fn kamu_node_run_flight_sql_server_e2e_test(input: TokenStream) -> TokenStream {
    let harness_method = parse_str("run_flight_sql_server").unwrap();

    kamu_node_e2e_test_impl(&harness_method, input)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Implementations
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn kamu_node_e2e_test_impl(harness_method: &Ident, input: TokenStream) -> TokenStream {
    let InputArgs {
        storage,
        fixture,
        options,
        extra_test_groups,
        repo_type,
    } = parse_macro_input!(input as InputArgs);

    let mut test_function_name = fixture.segments.last().unwrap().ident.clone();

    let options = options.unwrap_or_else(|| parse_str("Options::default()").unwrap());
    let repo_type = repo_type.unwrap_or_else(|| parse_str("local_fs").unwrap());
    test_function_name = syn::Ident::new(
        &format!("{test_function_name}_{repo_type}"),
        test_function_name.span(),
    );

    let extra_test_groups = if let Some(extra_test_groups) = extra_test_groups {
        parse_str(extra_test_groups.value().as_str()).unwrap()
    } else {
        quote! {}
    };

    let output = match storage.to_string().as_str() {
        "postgres" => quote! {
            #[test_group::group(e2e, database, postgres, #extra_test_groups)]
            #[test_log::test(sqlx::test(migrator = "database_common::POSTGRES_MIGRATOR"))]
            async fn #test_function_name (pg_pool: sqlx::PgPool) {
                KamuNodeApiServerHarness::postgres(&pg_pool, #options, &#repo_type)
                    . #harness_method ( #fixture )
                    .await;
            }
        },
        "sqlite" => quote! {
            #[test_group::group(e2e, database, sqlite, #extra_test_groups)]
            #[test_log::test(sqlx::test(migrator = "database_common::SQLITE_MIGRATOR"))]
            async fn #test_function_name (sqlite_pool: sqlx::SqlitePool) {
                KamuNodeApiServerHarness::sqlite(&sqlite_pool, #options, &#repo_type)
                    . #harness_method ( #fixture )
                    .await;
            }
        },
        unexpected => {
            panic!(
                "Unexpected E2E test storage: \"{unexpected}\"!\nAllowable values: \"postgres\" \
                 and \"sqlite\"."
            );
        }
    };

    output.into()
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Helpers
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct InputArgs {
    pub storage: Ident,
    pub fixture: Path,
    pub options: Option<Expr>,
    pub repo_type: Option<Ident>,
    pub extra_test_groups: Option<LitStr>,
}

impl Parse for InputArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut storage = None;
        let mut fixture = None;
        let mut options = None;
        let mut repo_type = None;
        let mut extra_test_groups = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "storage" => {
                    let value: Ident = input.parse()?;

                    storage = Some(value);
                }
                "fixture" => {
                    let value: Path = input.parse()?;

                    fixture = Some(value);
                }
                "options" => {
                    let value: Expr = input.parse()?;

                    options = Some(value);
                }
                "repo_type" => {
                    let value: Ident = input.parse()?;

                    repo_type = Some(value);
                }
                "extra_test_groups" => {
                    let value: LitStr = input.parse()?;

                    extra_test_groups = Some(value);
                }
                unexpected_key => panic!(
                    "Unexpected key: {unexpected_key}\nAllowable values: \"storage\", \
                     \"fixture\", \"options\", \"repo_type\" and \"extra_test_groups\"."
                ),
            };

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let Some(storage) = storage else {
            panic!("Mandatory parameter \"storage\" not found");
        };

        let Some(fixture) = fixture else {
            panic!("Mandatory parameter \"fixture\" not found");
        };

        Ok(InputArgs {
            storage,
            fixture,
            options,
            repo_type,
            extra_test_groups,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
