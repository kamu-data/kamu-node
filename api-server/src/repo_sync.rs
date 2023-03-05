use std::sync::Arc;

use kamu::domain::*;
use opendatafabric::RepositoryName;

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn repo_sync_loop(catalog: dill::Catalog, repo_name: RepositoryName) {
    let _panic_guard = KillProcessOnPanic;
    loop {
        {
            let span = tracing::span!(tracing::Level::DEBUG, "Synchronizing repository");
            let _span_guard = span.enter();
            sync_all_from_repo(
                catalog.get_one().unwrap(),
                catalog.get_one().unwrap(),
                catalog.get_one().unwrap(),
                &repo_name,
            )
            .await
            .expect("Aborting on sync error");
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub async fn sync_all_from_repo(
    local_repo: Arc<dyn LocalDatasetRepository>,
    search_svc: Arc<dyn SearchService>,
    sync_svc: Arc<dyn SyncService>,
    repo_name: &RepositoryName,
) -> Result<(), SyncError> {
    use futures::TryStreamExt;

    // TODO: This will hit the search limit
    let datasets_to_sync = search_svc
        .search(
            None,
            SearchOptions {
                repository_names: vec![repo_name.clone()],
            },
        )
        .await
        .int_err()?
        .datasets;

    let local_datasets_to_delete: Vec<_> = local_repo
        .get_all_datasets()
        .try_filter(|hdl| {
            std::future::ready(!datasets_to_sync.iter().any(|rn| *rn.dataset() == hdl.name))
        })
        .try_collect()
        .await?;

    if !local_datasets_to_delete.is_empty() {
        tracing::info!(
            "Deleting datasets that are no longer present in repo: {:?}",
            local_datasets_to_delete
        );

        // TODO: Handle dangling dependency issue
        for hdl in &local_datasets_to_delete {
            local_repo
                .delete_dataset(&hdl.as_local_ref())
                .await
                .int_err()?;
        }
    }

    tracing::debug!("Syncing {} datasets", datasets_to_sync.len());

    let sync_results = sync_svc
        .sync_multi(
            &mut datasets_to_sync
                .iter()
                .map(|r| (r.as_any_ref(), r.dataset().as_any_ref())),
            SyncOptions {
                trust_source: Some(true),
                create_if_not_exists: true,
                force: false,
            },
            None,
        )
        .await;

    let mut up_to_date = 0;
    let mut updated = 0;
    let mut diverged = Vec::new();

    for sync_res in sync_results {
        match sync_res.result {
            Ok(r) => match r {
                SyncResult::UpToDate { .. } => up_to_date += 1,
                SyncResult::Updated { .. } => updated += 1,
            },
            Err(SyncError::DatasetsDiverged { .. }) => diverged.push((sync_res.src, sync_res.dst)),
            Err(e) => {
                tracing::error!("Failed to sync dataset {}: {}", sync_res.src, e);
                return Err(e);
            }
        }
    }

    if !diverged.is_empty() {
        tracing::info!("Force re-syncing datasets: {:?}", diverged);

        // TODO: Handle dangling dependency issue
        for (_, local) in &diverged {
            local_repo
                .delete_dataset(&local.as_local_ref().unwrap())
                .await
                .int_err()?;
        }

        let sync_results = sync_svc
            .sync_multi(&mut diverged.iter().cloned(), SyncOptions::default(), None)
            .await;

        for sync_res in sync_results {
            match sync_res.result {
                Ok(_) => updated += 1,
                Err(e) => {
                    tracing::error!("Failed to re-sync dataset {}: {}", sync_res.src, e);
                    return Err(e);
                }
            }
        }
    }

    if updated > 0 || !diverged.is_empty() {
        tracing::info!(
            "Sync result up-to-date: {}, updated: {}, re-synced: {}",
            up_to_date,
            updated,
            diverged.len(),
        );
    } else {
        tracing::debug!(
            "Sync result up-to-date: {}, updated: {}, re-synced: {}",
            up_to_date,
            updated,
            diverged.len(),
        );
    }

    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////

struct KillProcessOnPanic;

impl Drop for KillProcessOnPanic {
    fn drop(&mut self) {
        if std::thread::panicking() {
            std::process::exit(1);
        }
    }
}
