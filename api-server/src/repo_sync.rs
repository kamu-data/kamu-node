use std::sync::Arc;

use kamu::{
    domain::{MetadataRepository, SyncError, SyncOptions, SyncResult, SyncService},
    infra::RepositoryFactory,
};
use opendatafabric::{RemoteDatasetName, RepositoryName};

pub fn repo_sync_loop(catalog: dill::Catalog, repo_name: &RepositoryName) {
    let _panic_guard = KillProcessOnPanic;
    loop {
        {
            let span = tracing::span!(tracing::Level::DEBUG, "Synchronizing repository");
            let _span_guard = span.enter();
            sync_all_from_repo(
                catalog.get_one().unwrap(),
                catalog.get_one().unwrap(),
                catalog.get_one().unwrap(),
                repo_name,
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
    repo_name: &RepositoryName,
) -> Result<(), SyncError> {
    let datasets_to_sync =
        get_all_remote_datasets(metadata_repo.as_ref(), repo_factory.as_ref(), repo_name)?;

    let local_datasets_to_delete: Vec<_> = metadata_repo
        .get_all_datasets()
        .filter(|hdl| !datasets_to_sync.iter().any(|rn| rn.dataset() == hdl.name))
        .collect();

    if !local_datasets_to_delete.is_empty() {
        tracing::info!(
            "Deleting datasets that are no longer present in repo: {:?}",
            local_datasets_to_delete
        );

        // TODO: Handle dangling dependency issue
        for hdl in &local_datasets_to_delete {
            metadata_repo.delete_dataset(&hdl.as_local_ref()).unwrap();
        }
    }

    tracing::debug!("Syncing {} datasets", datasets_to_sync.len());

    let sync_results = sync_svc.sync_from_multi(
        &mut datasets_to_sync
            .iter()
            .map(|r| (r.as_remote_ref(), r.dataset())),
        SyncOptions {},
        None,
    );

    let mut up_to_date = 0;
    let mut updated = 0;
    let mut diverged = Vec::new();

    for ((remote, local), res) in sync_results {
        match res {
            Ok(r) => match r {
                SyncResult::UpToDate { .. } => up_to_date += 1,
                SyncResult::Updated { .. } => updated += 1,
            },
            Err(SyncError::DatasetsDiverged { .. }) => diverged.push((remote, local)),
            Err(e) => {
                tracing::error!("Failed to sync dataset {}: {}", remote, e);
                return Err(e);
            }
        }
    }

    if !diverged.is_empty() {
        tracing::info!("Force re-syncing datasets: {:?}", diverged);

        // TODO: Handle dangling dependency issue
        for (_, local) in &diverged {
            metadata_repo.delete_dataset(&local.as_local_ref()).unwrap();
        }

        let sync_results =
            sync_svc.sync_from_multi(&mut diverged.iter().cloned(), SyncOptions {}, None);

        for ((remote, _), res) in sync_results {
            match res {
                Ok(_) => updated += 1,
                Err(e) => {
                    tracing::error!("Failed to re-sync dataset {}: {}", remote, e);
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
    repo_name: &RepositoryName,
) -> Result<Vec<RemoteDatasetName>, SyncError> {
    let repo = metadata_repo.get_repository(repo_name).unwrap();
    let repo_client = repo_factory.get_repository_client(&repo).unwrap();

    let rc = repo_client.lock().unwrap();

    // TODO: This will hit the search limit
    let res = rc.search(None)?;

    // TODO: Avoid the need for re-mapping names
    Ok(res
        .datasets
        .into_iter()
        .map(|r| RemoteDatasetName::new(repo_name, r.account().as_ref(), &r.dataset()))
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
