// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use database_common::NoOpDatabasePlugin;
use dill::*;
use email_gateway::FakeEmailSender;
use kamu::domain::{DidGeneratorDefault, ServerUrlConfig, TenancyConfig};
use kamu::DatasetStorageUnitLocalFs;
use kamu_accounts::{
    CurrentAccountSubject,
    JwtAuthenticationConfig,
    PredefinedAccountsConfig,
    DEFAULT_ACCOUNT_NAME_STR,
    DUMMY_EMAIL_ADDRESS,
};
use kamu_accounts_inmem::{InMemoryAccessTokenRepository, InMemoryAccountRepository};
use kamu_accounts_services::{
    AccessTokenServiceImpl,
    AuthenticationServiceImpl,
    LoginPasswordAuthProvider,
    PredefinedAccountsRegistrator,
};
use kamu_api_server::{FlowProgressNotifier, FLOW_FAILED_SUBJECT};
use kamu_datasets::{DatasetEntry, DatasetEntryRepository};
use kamu_datasets_inmem::InMemoryDatasetEntryRepository;
use kamu_datasets_services::DatasetEntryServiceImpl;
use kamu_flow_system::{
    Flow,
    FlowAgentConfig,
    FlowError,
    FlowEventStore,
    FlowID,
    FlowKey,
    FlowOutcome,
    FlowProgressMessage,
    FlowResult,
    FlowStartConditionExecutor,
    FlowTriggerAutoPolling,
};
use kamu_flow_system_inmem::InMemoryFlowEventStore;
use kamu_flow_system_services::{
    FlowQueryServiceImpl,
    MESSAGE_PRODUCER_KAMU_FLOW_PROGRESS_SERVICE,
};
use kamu_task_system::{TaskError, TaskID, TaskOutcome, TaskResult};
use messaging_outbox::{register_message_dispatcher, Outbox, OutboxExt, OutboxImmediateImpl};
use odf::DatasetID;
use tempfile::TempDir;
use time_source::SystemTimeSourceDefault;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test_log::test(tokio::test)]
async fn test_failed_flow_sends_email() {
    let harness = FlowProgressNotifierHarness::new().await;

    let dataset_id = odf::DatasetID::new_seeded_ed25519(b"test-dataset");
    let dataset_name = odf::DatasetName::new_unchecked("test-dataset");

    harness.make_dataset(&dataset_id, &dataset_name).await;
    harness.send_flow_failed(&dataset_id).await;

    let emails = harness.fake_email_sender.get_recorded_emails();
    assert_eq!(emails.len(), 1);

    let flow_failed_email = emails.first().unwrap();
    assert_eq!(
        flow_failed_email.recipient.as_ref(),
        DUMMY_EMAIL_ADDRESS.as_ref()
    );
    assert_eq!(
        flow_failed_email.subject,
        format!("{FLOW_FAILED_SUBJECT}: test-dataset")
    );
    assert!(flow_failed_email
        .body
        .contains("href=\"http://platform.example.com/test-dataset/flow-details/0/history\""));
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test_log::test(tokio::test)]
async fn test_success_flow_gives_no_emails() {
    let harness = FlowProgressNotifierHarness::new().await;

    let dataset_id = odf::DatasetID::new_seeded_ed25519(b"test-dataset");
    let dataset_name = odf::DatasetName::new_unchecked("test-dataset");

    harness.make_dataset(&dataset_id, &dataset_name).await;
    harness.send_flow_success(&dataset_id).await;

    let emails = harness.fake_email_sender.get_recorded_emails();
    assert_eq!(emails.len(), 0);
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct FlowProgressNotifierHarness {
    _tempdir: TempDir,
    catalog: Catalog,
    outbox: Arc<dyn Outbox>,
    fake_email_sender: Arc<FakeEmailSender>,
}

impl FlowProgressNotifierHarness {
    async fn new() -> Self {
        let mut b = dill::CatalogBuilder::new();

        let tempdir = tempfile::tempdir().unwrap();
        let datasets_dir = tempdir.path().join("datasets");
        std::fs::create_dir(&datasets_dir).unwrap();

        b.add::<FlowProgressNotifier>()
            .add_value(TenancyConfig::SingleTenant)
            .add_builder(
                messaging_outbox::OutboxImmediateImpl::builder()
                    .with_consumer_filter(messaging_outbox::ConsumerFilter::AllConsumers),
            )
            .bind::<dyn Outbox, OutboxImmediateImpl>()
            .add::<FakeEmailSender>()
            // TODO: use mocks to avoid this boilerplate, but it's waiting for kamu-cli#1010
            .add::<FlowQueryServiceImpl>()
            .add::<InMemoryFlowEventStore>()
            .add::<DatasetEntryServiceImpl>()
            .add::<InMemoryDatasetEntryRepository>()
            .add::<DidGeneratorDefault>()
            .add::<SystemTimeSourceDefault>()
            .add_builder(DatasetStorageUnitLocalFs::builder().with_root(datasets_dir))
            .bind::<dyn odf::DatasetStorageUnit, DatasetStorageUnitLocalFs>()
            .add_value(CurrentAccountSubject::new_test())
            .add_value(FlowAgentConfig::new(
                TimeDelta::seconds(1),
                TimeDelta::minutes(1),
            ))
            .add::<InMemoryAccountRepository>()
            .add::<AuthenticationServiceImpl>()
            .add::<AccessTokenServiceImpl>()
            .add::<InMemoryAccessTokenRepository>()
            .add::<PredefinedAccountsRegistrator>()
            .add::<LoginPasswordAuthProvider>()
            .add_value(PredefinedAccountsConfig::single_tenant())
            .add_value(JwtAuthenticationConfig::default())
            .add_value(ServerUrlConfig::new_test(None));

        NoOpDatabasePlugin::init_database_components(&mut b);

        register_message_dispatcher::<FlowProgressMessage>(
            &mut b,
            MESSAGE_PRODUCER_KAMU_FLOW_PROGRESS_SERVICE,
        );

        let catalog = b.build();

        init_on_startup::run_startup_jobs(&catalog).await.unwrap();

        let outbox = catalog.get_one().unwrap();
        let fake_email_sender = catalog.get_one().unwrap();

        Self {
            _tempdir: tempdir,
            catalog,
            outbox,
            fake_email_sender,
        }
    }

    async fn make_dataset(&self, dataset_id: &DatasetID, dataset_name: &odf::DatasetName) {
        let dataset_entry_repo = self
            .catalog
            .get_one::<dyn DatasetEntryRepository>()
            .unwrap();

        dataset_entry_repo
            .save_dataset_entry(&DatasetEntry {
                id: dataset_id.clone(),
                owner_id: odf::AccountID::new_seeded_ed25519(DEFAULT_ACCOUNT_NAME_STR.as_bytes()),
                name: dataset_name.clone(),
                created_at: Utc::now(),
            })
            .await
            .unwrap();
    }

    async fn send_flow_success(&self, dataset_id: &DatasetID) {
        let flow_event_store = self.catalog.get_one::<dyn FlowEventStore>().unwrap();
        let flow_id = flow_event_store.new_flow_id().await.unwrap();

        let (mut flow, task_id) = self.create_and_prepare_flow(dataset_id, flow_id);

        flow.on_task_finished(Utc::now(), task_id, TaskOutcome::Success(TaskResult::Empty))
            .unwrap();

        flow.save(flow_event_store.as_ref()).await.unwrap();

        self.outbox
            .post_message(
                MESSAGE_PRODUCER_KAMU_FLOW_PROGRESS_SERVICE,
                FlowProgressMessage::finished(
                    Utc::now(),
                    flow.flow_id,
                    FlowOutcome::Success(FlowResult::Empty),
                ),
            )
            .await
            .unwrap();
    }

    async fn send_flow_failed(&self, dataset_id: &DatasetID) {
        let flow_event_store = self.catalog.get_one::<dyn FlowEventStore>().unwrap();
        let flow_id = flow_event_store.new_flow_id().await.unwrap();

        let (mut flow, task_id) = self.create_and_prepare_flow(dataset_id, flow_id);

        flow.on_task_finished(Utc::now(), task_id, TaskOutcome::Failed(TaskError::Empty))
            .unwrap();

        flow.save(flow_event_store.as_ref()).await.unwrap();

        self.outbox
            .post_message(
                MESSAGE_PRODUCER_KAMU_FLOW_PROGRESS_SERVICE,
                FlowProgressMessage::finished(
                    Utc::now(),
                    flow.flow_id,
                    FlowOutcome::Failed(FlowError::Failed),
                ),
            )
            .await
            .unwrap();
    }

    fn create_and_prepare_flow(&self, dataset_id: &DatasetID, flow_id: FlowID) -> (Flow, TaskID) {
        let mut flow = Flow::new(
            Utc::now(),
            flow_id,
            FlowKey::dataset(
                dataset_id.clone(),
                kamu_flow_system::DatasetFlowType::Ingest,
            ),
            kamu_flow_system::FlowTriggerType::AutoPolling(FlowTriggerAutoPolling {
                trigger_time: Utc::now(),
            }),
            None,
        );

        flow.schedule_for_activation(Utc::now(), Utc::now())
            .unwrap();

        let task_id = TaskID::new(1);
        flow.set_relevant_start_condition(
            Utc::now(),
            kamu_flow_system::FlowStartCondition::Executor(FlowStartConditionExecutor { task_id }),
        )
        .unwrap();

        flow.on_task_scheduled(Utc::now(), task_id).unwrap();
        flow.on_task_running(Utc::now(), task_id).unwrap();

        (flow, task_id)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
