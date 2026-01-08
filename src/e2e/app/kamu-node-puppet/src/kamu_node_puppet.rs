// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::path::{Path, PathBuf};
use std::{ffi, fs};

use test_utils::LocalS3Server;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ExecuteCommandResult = assert_cmd::assert::Assert;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, PartialEq)]
pub enum RepositoryType {
    #[default]
    LocalFs,
    S3,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct KamuNodePuppet {
    workspace_path: PathBuf,
    temp_dir: Option<tempfile::TempDir>,
    _s3_server: Option<LocalS3Server>,
}

impl KamuNodePuppet {
    pub fn new<P: Into<PathBuf>>(workspace_path: P) -> Self {
        let workspace_path = workspace_path.into();

        Self {
            workspace_path,
            temp_dir: None,
            _s3_server: None,
        }
    }

    pub async fn new_workspace_tmp_with(options: NewWorkspaceOptions) -> Self {
        use std::fmt::Write;

        let temp_dir = tempfile::tempdir().unwrap();

        let s3_server = if options.repo_type == RepositoryType::S3 {
            Some(LocalS3Server::new().await)
        } else {
            None
        };

        if let Some(mut config) = options.kamu_api_server_config {
            let repo_path = match options.repo_type {
                RepositoryType::LocalFs => {
                    let repo = temp_dir.path().join("datasets");
                    fs::create_dir(repo.as_path()).unwrap();
                    repo.display().to_string()
                }
                RepositoryType::S3 => s3_server.as_ref().unwrap().url.to_string(),
            };

            write!(
                &mut config,
                "{}",
                indoc::formatdoc!(
                    r#"
                    repo:
                      repoUrl: {repo_path}
                    auth:
                      providers:
                        - kind: password
                          accounts:
                            - accountName: kamu
                              password: kamu.dev
                              email: kamu@example.com
                              properties: [ admin, canProvisionAccounts ]
                            - accountName: molecule
                              password: molecule.dev
                              email: molecule@example.com
                              properties: [ admin, canProvisionAccounts ]
                        - kind: github
                          clientId: FOO
                          clientSecret: BAR
                    "#
                )
            )
            .unwrap();

            fs::write(temp_dir.path().join("config.yaml"), config).unwrap();
        }

        let inst = Self::new(temp_dir.path());
        let inst = Self {
            temp_dir: Some(temp_dir),
            _s3_server: s3_server,
            ..inst
        };

        // TODO: Reconsider, perhaps use dotenv as a way to propagate vars per run
        unsafe {
            for (env_name, env_value) in options.env_vars {
                std::env::set_var(env_name, env_value);
            }
        }

        inst
    }

    pub fn workspace_path(&self) -> &Path {
        &self.workspace_path
    }

    pub fn get_e2e_output_data_path(&self) -> PathBuf {
        let temp_dir = self.temp_dir.as_ref().unwrap().path();

        temp_dir.join("e2e-output-data.txt")
    }

    pub async fn execute<I, S>(&self, cmd: I) -> ExecuteCommandResult
    where
        I: IntoIterator<Item = S>,
        S: AsRef<ffi::OsStr>,
    {
        self.execute_impl(cmd, None).await
    }

    async fn execute_impl<I, S>(
        &self,
        cmd: I,
        maybe_env: Option<Vec<(&ffi::OsStr, &ffi::OsStr)>>,
    ) -> ExecuteCommandResult
    where
        I: IntoIterator<Item = S>,
        S: AsRef<ffi::OsStr>,
    {
        #[allow(deprecated)]
        let mut command = assert_cmd::Command::cargo_bin("kamu-api-server").unwrap();

        if let Some(env_vars) = maybe_env {
            for (name, value) in env_vars {
                command.env(name, value);
            }
        }

        command.env("RUST_LOG", "info,sqlx=debug");
        command.current_dir(self.workspace_path.clone());
        command.args(cmd);

        tokio::task::spawn_blocking(move || command.assert())
            .await
            .unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct NewWorkspaceOptions {
    pub repo_type: RepositoryType,
    pub kamu_api_server_config: Option<String>,
    pub env_vars: Vec<(String, String)>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
