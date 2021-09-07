use std::sync::Arc;

use kamu::{
    domain::{MetadataRepository, SyncError, SyncOptions, SyncResult, SyncService},
    infra::RepositoryFactory,
};
use opendatafabric::{DatasetRefBuf, RepositoryID};

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
    // TODO: This will hit the search limit
    let datasets_to_sync: Vec<_> = {
        let repo = metadata_repo.get_repository(repo_id).unwrap();
        let repo_client = repo_factory.get_repository_client(&repo).unwrap();

        let rc = repo_client.lock().unwrap();
        let res = rc.search(None)?;
        res.datasets
            .into_iter()
            .map(|r| DatasetRefBuf::new(Some(repo_id), r.username(), r.local_id()))
            .collect()
    };

    tracing::debug!("Syncing {} datasets", datasets_to_sync.len());

    let sync_results = sync_svc.sync_from_multi(
        &mut datasets_to_sync.iter().map(|r| (r.as_ref(), r.local_id())),
        SyncOptions {},
        None,
    );

    let mut up_to_date = 0;
    let mut updated = 0;

    for ((id, _), res) in sync_results {
        match res {
            Ok(r) => match r {
                SyncResult::UpToDate { .. } => up_to_date += 1,
                SyncResult::Updated { .. } => updated += 1,
            },
            Err(e) => {
                tracing::error!("Failed to sync dataset {}: {}", id, e);
                return Err(e);
            }
        }
    }

    if updated > 0 {
        tracing::info!(
            "Sync result up-to-date: {}, updated: {}",
            up_to_date,
            updated
        );
    } else {
        tracing::debug!(
            "Sync result up-to-date: {}, updated: {}",
            up_to_date,
            updated
        );
    }

    Ok(())
}

struct KillProcessOnPanic;

impl Drop for KillProcessOnPanic {
    fn drop(&mut self) {
        if std::thread::panicking() {
            std::process::exit(1);
        }
    }
}
