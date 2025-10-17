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
use dill::{Catalog, component, interface, meta};
use email_gateway::EmailSender;
use internal_error::{InternalError, ResultIntoInternal};
use kamu_datasets::GetDatasetEntryError;
use kamu_flow_system as kamu_fs;
use messaging_outbox::{
    InitialConsumerBoundary,
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
#[interface(dyn MessageConsumerT<kamu_fs::FlowProcessLifecycleMessage>)]
#[meta(MessageConsumerMeta {
    consumer_name: MESSAGE_CONSUMER_KAMU_API_SERVER_FLOW_PROGRESS_NOTIFIER,
    feeding_producers: &[
        kamu_flow_system::MESSAGE_PRODUCER_KAMU_FLOW_PROCESS_STATE_PROJECTOR,
    ],
    delivery: MessageDeliveryMechanism::Transactional,
    initial_consumer_boundary: InitialConsumerBoundary::Latest,
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

    async fn notify_flow_failed(&self, flow_id: kamu_fs::FlowID) -> Result<(), InternalError> {
        // Load flow aggregate
        let flow_state = self.flow_query_service.get_flow(flow_id).await.int_err()?;

        let scope_type = flow_state.flow_binding.scope.scope_type();
        if scope_type == kamu_adapter_flow_dataset::FLOW_SCOPE_TYPE_DATASET {
            let dataset_scope =
                kamu_adapter_flow_dataset::FlowScopeDataset::new(&flow_state.flow_binding.scope);
            let dataset_id = dataset_scope.dataset_id();
            let dataset_entry = match self.dataset_entry_service.get_entry(&dataset_id).await {
                Ok(entry) => entry,
                Err(GetDatasetEntryError::NotFound(_)) => {
                    tracing::error!(
                        %flow_id, %dataset_id,
                        "Dataset not found within flow progress email handler"
                    );
                    return Ok(());
                }
                Err(GetDatasetEntryError::Internal(e)) => return Err(e),
            };

            // Owner account is needed for proper links as a minimum
            let owner_account = self
                .account_service
                .get_account_by_id(&dataset_entry.owner_id)
                .await
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
                    flow_state.flow_binding.flow_type.as_str(),
                    FlowFailureData {
                        id: flow_id,
                        // ToDo: Replace by the real fail error message
                        error: "Unknown",
                        started_at: flow_state
                            .timing
                            .running_since
                            .expect("Start time should be defined"),
                        occurred_at: flow_state
                            .timing
                            .last_attempt_finished_at
                            .expect("Finish time should be defined"),
                        primary_activation_cause: flow_state.primary_activation_cause(),
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
        } else {
            // TODO: system flows? webhook flows?
            tracing::warn!(
                flow_id = %flow_id,
                "Flow progress notifier received a message for a non-dataset flow: {}",
                scope_type
            );
        }

        Ok(())
    }

    async fn select_dataset_flow_recipient<'a>(
        &self,
        owner_account: &'a kamu_accounts::Account,
        flow_state: &'a kamu_fs::FlowState,
    ) -> Result<Cow<'a, kamu_accounts::Account>, InternalError> {
        if let kamu_fs::FlowActivationCause::Manual(m) = flow_state.primary_activation_cause()
            && m.initiator_account_id != owner_account.id
        {
            let initiator_account = self
                .account_service
                .get_account_by_id(&m.initiator_account_id)
                .await
                .expect("Account must be resolved");
            return Ok(Cow::Owned(initiator_account));
        }

        Ok(Cow::Borrowed(owner_account))
    }

    async fn render_dataset_flow_failure_email(
        &self,
        owner_account: &kamu_accounts::Account,
        dataset_entry: &kamu_datasets::DatasetEntry,
        flow_type: &str,
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

        let email = FlowFailedEmail {
            flow_type,
            dataset_alias: &dataset_alias,
            dataset_url: dataset_url.as_str(),
            trigger_instance: self
                .format_activation_cause(flow_failure_data.primary_activation_cause),
            failure_reason: flow_failure_data.error,
            start_time: start_time.as_str(),
            failure_time: failure_time.as_str(),
            flow_details_url: &flow_details_url,
        };

        email.render().int_err()
    }

    fn format_activation_cause(
        &self,
        flow_activation_cause: &kamu_fs::FlowActivationCause,
    ) -> &str {
        match flow_activation_cause {
            kamu_fs::FlowActivationCause::AutoPolling(_) => "Automatic",
            kamu_fs::FlowActivationCause::Manual(_) => "Manual",
            kamu_fs::FlowActivationCause::ResourceUpdate(_) => "Input Resource Updated",
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
impl MessageConsumerT<kamu_fs::FlowProcessLifecycleMessage> for FlowProgressNotifier {
    #[tracing::instrument(
        level = "debug",
        skip_all,
        name = "FlowProgressNotifier[FlowProcessLifecycleMessage]"
    )]
    async fn consume_message(
        &self,
        _: &Catalog,
        message: &kamu_fs::FlowProcessLifecycleMessage,
    ) -> Result<(), InternalError> {
        tracing::debug!(received_message = ?message, "Received flow progress message");
        match message {
            kamu_fs::FlowProcessLifecycleMessage::FailureRegistered(failed) => {
                self.notify_flow_failed(failed.flow_id).await
            }

            kamu_fs::FlowProcessLifecycleMessage::EffectiveStateChanged(_)
            | kamu_fs::FlowProcessLifecycleMessage::TriggerAutoStopped(_) => Ok(()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct FlowFailureData<'a> {
    id: kamu_fs::FlowID,
    primary_activation_cause: &'a kamu_fs::FlowActivationCause,
    started_at: DateTime<Utc>,
    occurred_at: DateTime<Utc>,
    error: &'a str,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Template)]
#[template(path = "flow-failed.html")]
struct FlowFailedEmail<'a> {
    flow_type: &'a str,
    dataset_alias: &'a str,
    dataset_url: &'a str,
    trigger_instance: &'a str,
    start_time: &'a str,
    failure_time: &'a str,
    failure_reason: &'a str,
    flow_details_url: &'a str,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
