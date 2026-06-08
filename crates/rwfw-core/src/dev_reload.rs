use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Snapshot {
    any: Option<SystemTime>,
    css: Option<SystemTime>,
}

pub fn spawn_watcher(roots: Vec<PathBuf>, tx: tokio::sync::broadcast::Sender<String>) {
    if roots.is_empty() {
        return;
    }

    tokio::spawn(async move {
        let mut last = snapshot_roots(&roots);
        let mut interval = tokio::time::interval(Duration::from_millis(700));

        loop {
            interval.tick().await;
            let current = snapshot_roots(&roots);
            if current == last {
                continue;
            }

            let token = if current.css != last.css {
                "css"
            } else {
                "reload"
            };
            let _ = tx.send(token.to_string());
            last = current;
        }
    });
}

fn snapshot_roots(roots: &[PathBuf]) -> Snapshot {
    let mut snapshot = Snapshot::default();
    for root in roots {
        visit(root, &mut snapshot);
    }
    snapshot
}

fn visit(path: &Path, snapshot: &mut Snapshot) {
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };

    if metadata.is_file() {
        if let Ok(modified) = metadata.modified() {
            update_latest(&mut snapshot.any, modified);
            if path.extension().is_some_and(|extension| extension == "css") {
                update_latest(&mut snapshot.css, modified);
            }
        }
        return;
    }

    if !metadata.is_dir() {
        return;
    }

    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, ".git" | "target" | "node_modules"))
        {
            continue;
        }
        visit(&path, snapshot);
    }
}

fn update_latest(slot: &mut Option<SystemTime>, candidate: SystemTime) {
    if slot.is_none_or(|current| candidate > current) {
        *slot = Some(candidate);
    }
}
