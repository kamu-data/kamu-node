// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::borrow::Cow;
use std::sync::Arc;

use askama::Template;
use chrono::{DateTime, Utc};
use dill::{component, interface, meta, Catalog};
use email_gateway::EmailSender;
use internal_error::{InternalError, ResultIntoInternal};
use kamu_flow_system as kamu_fs;
use messaging_outbox::{
    MessageConsumer,
    MessageConsumerMeta,
    MessageConsumerT,
    MessageDeliveryMechanism,
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub const FLOW_FAILED_SUBJECT: &str = "Kamu Flow Run Failed";

pub const MESSAGE_CONSUMER_KAMU_API_SERVER_FLOW_PROGRESS_NOTIFIER: &str =
    "dev.kamu.api-server.FlowProgressNotifier";

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct FlowProgressNotifier {
    email_sender: Arc<dyn EmailSender>,
    flow_query_service: Arc<dyn kamu_fs::FlowQueryService>,
    dataset_entry_service: Arc<dyn kamu_datasets::DatasetEntryService>,
    account_service: Arc<dyn kamu_accounts::AccountService>,
    server_url_config: Arc<kamu::domain::ServerUrlConfig>,
    tenancy_config: Arc<kamu::domain::TenancyConfig>,
}

#[component(pub)]
#[interface(dyn MessageConsumer)]
#[interface(dyn MessageConsumerT<kamu_fs::FlowProgressMessage>)]
#[meta(MessageConsumerMeta {
    consumer_name: MESSAGE_CONSUMER_KAMU_API_SERVER_FLOW_PROGRESS_NOTIFIER,
    feeding_producers: &[
        kamu_flow_system_services::MESSAGE_PRODUCER_KAMU_FLOW_PROGRESS_SERVICE,
    ],
    delivery: MessageDeliveryMechanism::Transactional,
})]
impl FlowProgressNotifier {
    pub fn new(
        email_sender: Arc<dyn EmailSender>,
        flow_query_service: Arc<dyn kamu_fs::FlowQueryService>,
        dataset_entry_service: Arc<dyn kamu_datasets::DatasetEntryService>,
        account_service: Arc<dyn kamu_accounts::AccountService>,
        server_url_config: Arc<kamu::domain::ServerUrlConfig>,
        tenancy_config: Arc<kamu::domain::TenancyConfig>,
    ) -> Self {
        Self {
            email_sender,
            flow_query_service,
            dataset_entry_service,
            account_service,
            server_url_config,
            tenancy_config,
        }
    }

    async fn notify_flow_failed(
        &self,
        flow_id: kamu_fs::FlowID,
        flow_error: &kamu_fs::FlowError,
    ) -> Result<(), InternalError> {
        // Load flow aggregate
        let flow_state = self.flow_query_service.get_flow(flow_id).await.int_err()?;
        match &flow_state.flow_key {
            // Dataset flow => notify owner or initiator
            kamu_fs::FlowKey::Dataset(fk_dataset) => {
                // Load related dataset entry
                let dataset_entry = self
                    .dataset_entry_service
                    .get_entry(&fk_dataset.dataset_id)
                    .await
                    .int_err()?;

                // Owner account is needed for proper links as a minimum
                let owner_account = self
                    .account_service
                    .account_by_id(&dataset_entry.owner_id)
                    .await
                    .int_err()?
                    .unwrap();

                // Select recipient: manual different launched person or owner
                let recipient_account = self
                    .select_dataset_flow_recipient(&owner_account, &flow_state)
                    .await?;

                // Render email
                let rendered_email = self
                    .render_dataset_flow_failure_email(
                        &owner_account,
                        &dataset_entry,
                        fk_dataset,
                        FlowFailureData {
                            id: flow_id,
                            error: flow_error,
                            started_at: flow_state
                                .timing
                                .running_since
                                .expect("Start time should be defined"),
                            occurred_at: flow_state
                                .timing
                                .finished_at
                                .expect("Finish time should be defined"),
                            primary_trigger_type: flow_state.primary_trigger(),
                        },
                    )
                    .await?;

                // Format subject
                let email_subject = format!(
                    "{}: {}",
                    FLOW_FAILED_SUBJECT,
                    self.format_dataset_alias(&owner_account, &dataset_entry)
                );

                // Deliver email to selected recipient
                self.email_sender
                    .send_email(&recipient_account.email, &email_subject, &rendered_email)
                    .await
                    .int_err()?;
            }

            // TODO: notify admin(s) about system flow failure?
            kamu_fs::FlowKey::System(_) => {}
        }

        Ok(())
    }

    async fn select_dataset_flow_recipient<'a>(
        &self,
        owner_account: &'a kamu_accounts::Account,
        flow_state: &'a kamu_fs::FlowState,
    ) -> Result<Cow<'a, kamu_accounts::Account>, InternalError> {
        if let kamu_fs::FlowTriggerType::Manual(m) = flow_state.primary_trigger()
            && m.initiator_account_id != owner_account.id
        {
            let initiator_account = self
                .account_service
                .account_by_id(&m.initiator_account_id)
                .await?
                .expect("Account must be resolved");
            return Ok(Cow::Owned(initiator_account));
        }

        Ok(Cow::Borrowed(owner_account))
    }

    async fn render_dataset_flow_failure_email(
        &self,
        owner_account: &kamu_accounts::Account,
        dataset_entry: &kamu_datasets::DatasetEntry,
        flow_key_dataset: &kamu_fs::FlowKeyDataset,
        flow_failure_data: FlowFailureData<'_>,
    ) -> Result<String, InternalError> {
        let dataset_alias = self.format_dataset_alias(owner_account, dataset_entry);
        let dataset_url = self.format_dataset_url(owner_account, dataset_entry);
        let flow_details_url =
            self.format_flow_details_url(owner_account, dataset_entry, flow_failure_data.id);

        let failure_time = flow_failure_data
            .occurred_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let start_time = flow_failure_data
            .started_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let failure_reason = self
            .format_flow_failure_reason(flow_failure_data.error)
            .await?;

        let email = FlowFailedEmail {
            flow_type: self.format_dataset_flow_type(flow_key_dataset),
            dataset_alias: &dataset_alias,
            dataset_url: dataset_url.as_str(),
            trigger_type: self.format_flow_trigger_type(flow_failure_data.primary_trigger_type),
            failure_reason: &failure_reason,
            start_time: start_time.as_str(),
            failure_time: failure_time.as_str(),
            flow_details_url: &flow_details_url,
        };

        email.render().int_err()
    }

    fn format_dataset_flow_type(&self, fk_dataset: &kamu_fs::FlowKeyDataset) -> &str {
        match fk_dataset.flow_type {
            kamu_fs::DatasetFlowType::Ingest | kamu_fs::DatasetFlowType::ExecuteTransform => {
                "Update"
            }
            kamu_fs::DatasetFlowType::HardCompaction => "Compact",
            kamu_fs::DatasetFlowType::Reset => "Reset",
        }
    }

    fn format_flow_trigger_type(&self, flow_trigger_type: &kamu_fs::FlowTriggerType) -> &str {
        match flow_trigger_type {
            kamu_fs::FlowTriggerType::AutoPolling(_) => "Automatic",
            kamu_fs::FlowTriggerType::Manual(_) => "Manual",
            kamu_fs::FlowTriggerType::Push(_) => "Data Push",
            kamu_fs::FlowTriggerType::InputDatasetFlow(_) => "Input Dataset Flow",
        }
    }

    async fn format_flow_failure_reason(
        &self,
        flow_error: &kamu_fs::FlowError,
    ) -> Result<String, InternalError> {
        match flow_error {
            kamu_fs::FlowError::Failed => Ok("Unknown".to_string()),
            kamu_fs::FlowError::InputDatasetCompacted(c) => {
                let input_dataset_entry = self
                    .dataset_entry_service
                    .get_entry(&c.dataset_id)
                    .await
                    .int_err()?;
                Ok(format!(
                    "Input dataset '{}' compacted",
                    input_dataset_entry.name
                ))
            }
            kamu_fs::FlowError::ResetHeadNotFound => Ok("Reset head not found".to_string()),
        }
    }

    fn format_dataset_alias(
        &self,
        owner_account: &kamu_accounts::Account,
        dataset_entry: &kamu_datasets::DatasetEntry,
    ) -> String {
        match self.tenancy_config.as_ref() {
            kamu::domain::TenancyConfig::SingleTenant => format!("{}", dataset_entry.name),
            kamu::domain::TenancyConfig::MultiTenant => {
                format!("{}/{}", owner_account.account_name, dataset_entry.name)
            }
        }
    }

    fn format_dataset_url(
        &self,
        owner_account: &kamu_accounts::Account,
        dataset_entry: &kamu_datasets::DatasetEntry,
    ) -> String {
        format!(
            "{}{}",
            self.server_url_config.protocols.base_url_platform,
            self.format_dataset_alias(owner_account, dataset_entry)
        )
    }

    fn format_flow_details_url(
        &self,
        owner_account: &kamu_accounts::Account,
        dataset_entry: &kamu_datasets::DatasetEntry,
        flow_id: kamu_fs::FlowID,
    ) -> String {
        format!(
            "{}/flow-details/{}/history",
            self.format_dataset_url(owner_account, dataset_entry),
            flow_id,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl MessageConsumer for FlowProgressNotifier {}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
impl MessageConsumerT<kamu_fs::FlowProgressMessage> for FlowProgressNotifier {
    #[tracing::instrument(
        level = "debug",
        skip_all,
        name = "FlowProgressNotifier[FlowProgressMessage]"
    )]
    async fn consume_message(
        &self,
        _: &Catalog,
        message: &kamu_fs::FlowProgressMessage,
    ) -> Result<(), InternalError> {
        tracing::debug!(received_message = ?message, "Received flow progress message");
        match message {
            kamu_fs::FlowProgressMessage::Finished(finished) => match &finished.outcome {
                kamu_fs::FlowOutcome::Failed(flow_error) => {
                    self.notify_flow_failed(finished.flow_id, flow_error).await
                }
                kamu_fs::FlowOutcome::Aborted | kamu_fs::FlowOutcome::Success(_) => Ok(()),
            },
            kamu_fs::FlowProgressMessage::Cancelled(_)
            | kamu_fs::FlowProgressMessage::Running(_)
            | kamu_fs::FlowProgressMessage::Scheduled(_) => Ok(()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct FlowFailureData<'a> {
    id: kamu_fs::FlowID,
    primary_trigger_type: &'a kamu_fs::FlowTriggerType,
    started_at: DateTime<Utc>,
    occurred_at: DateTime<Utc>,
    error: &'a kamu_fs::FlowError,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Template)]
#[template(path = "flow-failed.html")]
struct FlowFailedEmail<'a> {
    flow_type: &'a str,
    dataset_alias: &'a str,
    dataset_url: &'a str,
    trigger_type: &'a str,
    start_time: &'a str,
    failure_time: &'a str,
    failure_reason: &'a str,
    flow_details_url: &'a str,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
