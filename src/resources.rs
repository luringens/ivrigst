//! This module contains the [Resources] struct, which finds and watches the resources directory
//! containing models and shaders and provides functions to easily parse them into memory.
use anyhow::{anyhow, Context, Result};
use notify::{Event, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Receiver};
use std::{
    ffi, fs,
    io::Read,
    path::{Path, PathBuf},
};

/// Find and watches the resources directory containing models and shaders.
pub struct Resources {
    root_path: PathBuf,
    _watcher: notify::RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
}

impl Resources {
    /// Initializes a [Resources] struct, attempting to find the resources directory and beginning
    /// to watch for file changes.
    pub fn from_relative_exe_path(rel_path: &Path) -> Result<Resources> {
        let exe_file_name = ::std::env::current_exe().context("No exe filename")?;
        let exe_path = exe_file_name.parent().context("No exe parent")?;
        let root_path = exe_path.join(rel_path);

        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(&root_path, RecursiveMode::Recursive)?;

        Ok(Resources {
            root_path,
            _watcher: watcher,
            rx,
        })
    }

    /// Lists all changed files since last check.
    pub fn updated_paths(&self) -> Vec<PathBuf> {
        let mut events = Vec::new();
        match self.rx.try_recv() {
            Ok(Ok(Event {
                kind: notify::EventKind::Modify(_),
                mut paths,
                ..
            })) => events.append(&mut paths),
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(e) => eprintln!("File watch error: {:?}", e),
            _ => {}
        }
        events
    }

    /// Attempts to load the given text file.
    pub fn load_cstring(&self, resource_name: &str) -> Result<ffi::CString> {
        let mut file = fs::File::open(resource_name_to_path(&self.root_path, resource_name))
            .context("Failed to open resource file")?;

        // allocate buffer of the same size as file
        let mut buffer: Vec<u8> = Vec::with_capacity(file.metadata()?.len() as usize + 1);
        file.read_to_end(&mut buffer)?;

        // check for nul byte
        if buffer.iter().any(|i| *i == 0) {
            return Err(anyhow!("Resource contains NUL byte."));
        }

        Ok(unsafe { ffi::CString::from_vec_unchecked(buffer) })
    }

    /// Attempts to load the given obj file.
    pub fn load_model(&self, resource_name: &str) -> Result<tobj::Mesh> {
        let path = resource_name_to_path(&self.root_path, resource_name);
        let settings = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        };
        let (mut models, _) = tobj::load_obj(path, &settings)?;
        let model = models
            .pop()
            .ok_or_else(|| anyhow!("Obj file has no model!"))?;

        Ok(model.mesh)
    }

    /// Lists all models found in the root resource directory.
    pub fn list_models(&self) -> Vec<String> {
        std::fs::read_dir(&self.root_path)
            .and_then(|readdir| {
                readdir
                    .map(|entry| entry.map(|d| d.file_name().to_string_lossy().into_owned()))
                    .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_default()
            .into_iter()
            .filter(|entry| entry.ends_with(".obj"))
            .map(|file| file.trim_end_matches(".obj").to_owned())
            .collect()
    }
}

/// Joins a location string to a root directory path.
fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.split('/') {
        path = path.join(part);
    }

    path
}
