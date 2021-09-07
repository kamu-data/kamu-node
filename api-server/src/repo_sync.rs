use std::sync::Arc;

use kamu::{
    domain::{MetadataRepository, SyncError, SyncOptions, SyncResult, SyncService},
    infra::RepositoryFactory,
};
use opendatafabric::{DatasetID, DatasetIDBuf, DatasetRefBuf, RepositoryID};

pub fn repo_sync_loop(catalog: dill::Catalog, repo_id: &RepositoryID) {
    let _panic_guard = KillProcessOnPanic;
    loop {
        {
            let span = tracing::span!(tracing::Level::DEBUG, "Synchronizing repository");
            let _span_guard = span.enter();
            sync_all_from_repo(
                catalog.get_one().unwrap(),
                catalog.get_one().unwrap(),
                catalog.get_one().unwrap(),
                repo_id,
            )
            .expect("Aborting on sync error");
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

pub fn sync_all_from_repo(
    metadata_repo: Arc<dyn MetadataRepository>,
    repo_factory: Arc<RepositoryFactory>,
    sync_svc: Arc<dyn SyncService>,
    repo_id: &RepositoryID,
) -> Result<(), SyncError> {
    let datasets_to_sync =
        get_all_remote_datasets(metadata_repo.as_ref(), repo_factory.as_ref(), repo_id)?;

    let local_datasets_to_delete: Vec<DatasetIDBuf> = metadata_repo
        .get_all_datasets()
        .filter(|id| {
            for rem in &datasets_to_sync {
                if rem.local_id() == (id as &DatasetID) {
                    return false;
                }
            }
            true
        })
        .collect();

    if !local_datasets_to_delete.is_empty() {
        tracing::info!(
            "Deleting datasets that are no longer present in repo: {:?}",
            local_datasets_to_delete
        );

        // TODO: Handle dangling dependency issue
        for id in &local_datasets_to_delete {
            metadata_repo.delete_dataset(id).unwrap();
        }
    }

    tracing::debug!("Syncing {} datasets", datasets_to_sync.len());

    let sync_results = sync_svc.sync_from_multi(
        &mut datasets_to_sync.iter().map(|r| (r.as_ref(), r.local_id())),
        SyncOptions {},
        None,
    );

    let mut up_to_date = 0;
    let mut updated = 0;
    let mut diverged = Vec::new();

    for ((id, _), res) in sync_results {
        match res {
            Ok(r) => match r {
                SyncResult::UpToDate { .. } => up_to_date += 1,
                SyncResult::Updated { .. } => updated += 1,
            },
            Err(SyncError::DatasetsDiverged { .. }) => diverged.push(id),
            Err(e) => {
                tracing::error!("Failed to sync dataset {}: {}", id, e);
                return Err(e);
            }
        }
    }

    if !diverged.is_empty() {
        tracing::info!("Force re-syncing datasets: {:?}", diverged);

        // TODO: Handle dangling dependency issue
        for id in &diverged {
            metadata_repo.delete_dataset(id.local_id()).unwrap();
        }

        let sync_results = sync_svc.sync_from_multi(
            &mut diverged.iter().map(|r| (r.as_ref(), r.local_id())),
            SyncOptions {},
            None,
        );

        for ((id, _), res) in sync_results {
            match res {
                Ok(_) => updated += 1,
                Err(e) => {
                    tracing::error!("Failed to re-sync dataset {}: {}", id, e);
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

fn get_all_remote_datasets(
    metadata_repo: &dyn MetadataRepository,
    repo_factory: &RepositoryFactory,
    repo_id: &RepositoryID,
) -> Result<Vec<DatasetRefBuf>, SyncError> {
    let repo = metadata_repo.get_repository(repo_id).unwrap();
    let repo_client = repo_factory.get_repository_client(&repo).unwrap();

    let rc = repo_client.lock().unwrap();

    // TODO: This will hit the search limit
    let res = rc.search(None)?;

    Ok(res
        .datasets
        .into_iter()
        .map(|r| DatasetRefBuf::new(Some(repo_id), r.username(), r.local_id()))
        .collect())
}

struct KillProcessOnPanic;

impl Drop for KillProcessOnPanic {
    fn drop(&mut self) {
        if std::thread::panicking() {
            std::process::exit(1);
        }
    }
}
