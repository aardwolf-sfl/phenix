use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use crossbeam_channel::Receiver;
use notify::{RecommendedWatcher, Watcher};
use parking_lot::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VfsPath(Arc<PathBuf>);

impl VfsPath {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        path.as_ref().canonicalize().map(Arc::new).map(Self)
    }

    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

impl TryFrom<&Path> for VfsPath {
    type Error = io::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

pub struct Vfs {
    inner: Arc<Mutex<RecommendedWatcher>>,
    receiver: Receiver<(VfsPath, ChangeKind)>,
}

impl Vfs {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();

        let inner =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                Ok(event) => {
                    if let Some(kind) = ChangeKind::from_notify(&event.kind) {
                        let paths = event
                            .paths
                            .into_iter()
                            .filter_map(|path| VfsPath::new(path).ok());

                        for path in paths {
                            let _ = sender.send((path, kind));
                        }
                    }
                }
                Err(error) => eprintln!("error while watching a file: {:?}", error),
            })
            .expect("files watching is not available");

        Self {
            inner: Arc::new(Mutex::new(inner)),
            receiver,
        }
    }

    pub fn watch(&self, path: &VfsPath) {
        match self
            .inner
            .lock()
            .watch(path.as_path(), notify::RecursiveMode::Recursive)
        {
            Ok(_) => {}
            Err(error) => eprintln!("error when trying to watch a file: {:?}", error),
        }
    }

    pub fn recv(&self) -> Result<(VfsPath, ChangeKind), crossbeam_channel::RecvError> {
        self.receiver.recv()
    }

    pub fn try_recv(&self) -> Result<(VfsPath, ChangeKind), crossbeam_channel::TryRecvError> {
        self.receiver.try_recv()
    }
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

pub trait VfsWatcher {
    fn watch(&self, path: &VfsPath);
    fn on_change(&mut self, path: &VfsPath, kind: ChangeKind);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    Create,
    Modify,
    Remove,
}

impl ChangeKind {
    fn from_notify(event_kind: &notify::EventKind) -> Option<Self> {
        match event_kind {
            notify::EventKind::Create(_) => Some(ChangeKind::Create),
            notify::EventKind::Modify(_) => Some(ChangeKind::Modify),
            notify::EventKind::Remove(_) => Some(ChangeKind::Remove),
            _ => None,
        }
    }
}
