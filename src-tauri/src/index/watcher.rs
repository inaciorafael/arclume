use std::collections::HashSet;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use notify::{EventKind, RecursiveMode, Watcher};

use super::FileIndex;

const PERIODIC_RECONCILIATION: Duration = Duration::from_secs(60 * 60);
const ROOT_REFRESH: Duration = Duration::from_secs(2);

pub fn run(index: Arc<FileIndex>) {
    let (sender, receiver) = mpsc::channel();
    let Ok(mut watcher) = notify::recommended_watcher(sender) else {
        eprintln!("filesystem watcher is unavailable");
        return;
    };
    let mut watched = HashSet::new();
    sync_roots(&index, &mut watcher, &mut watched);
    index.reconcile("startup");
    let mut last_reconciliation = Instant::now();
    loop {
        match receiver.recv_timeout(ROOT_REFRESH) {
            Ok(Ok(event)) => {
                for path in event.paths {
                    match event.kind {
                        EventKind::Remove(_) => index.remove_path(&path),
                        EventKind::Create(_) | EventKind::Modify(_) => {
                            if path.exists() {
                                index.upsert_path(&path);
                            } else {
                                index.remove_path(&path);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Err(error)) => {
                eprintln!("filesystem watcher error; reconciling index: {error}");
                index.reconcile("watcher-error");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                sync_roots(&index, &mut watcher, &mut watched);
                if last_reconciliation.elapsed() >= PERIODIC_RECONCILIATION {
                    index.reconcile("periodic");
                    last_reconciliation = Instant::now();
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("filesystem watcher disconnected");
                break;
            }
        }
    }
}

fn sync_roots(
    index: &Arc<FileIndex>,
    watcher: &mut impl Watcher,
    watched: &mut HashSet<std::path::PathBuf>,
) {
    let desired: HashSet<_> = index.roots().into_iter().collect();
    for root in watched.difference(&desired).cloned().collect::<Vec<_>>() {
        let _ = watcher.unwatch(&root);
        watched.remove(&root);
    }
    for root in desired.difference(watched).cloned().collect::<Vec<_>>() {
        match watcher.watch(&root, RecursiveMode::Recursive) {
            Ok(()) => {
                watched.insert(root);
            }
            Err(error) => eprintln!("failed to watch {}: {error}", root.display()),
        }
    }
}
