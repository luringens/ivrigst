use anyhow::{anyhow, Context, Result};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::{
    ffi, fs,
    io::Read,
    path::{Path, PathBuf},
};

pub struct Resources {
    root_path: PathBuf,
    _watcher: notify::RecommendedWatcher,
    rx: Receiver<DebouncedEvent>,
}

impl Resources {
    pub fn from_relative_exe_path(rel_path: &Path) -> Result<Resources> {
        let exe_file_name = ::std::env::current_exe().context("No exe filename")?;
        let exe_path = exe_file_name.parent().context("No exe parent")?;
        let root_path = exe_path.join(rel_path);

        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_millis(250))?;
        watcher.watch(&root_path, RecursiveMode::Recursive)?;

        Ok(Resources {
            root_path,
            _watcher: watcher,
            rx,
        })
    }

    pub fn updated_paths(&self) -> Vec<PathBuf> {
        let mut events = Vec::new();
        match self.rx.try_recv() {
            Ok(DebouncedEvent::Write(path)) => events.push(path),
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(e) => eprintln!("File watch error: {:?}", e),
            _ => {}
        }
        events
    }

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

fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.split('/') {
        path = path.join(part);
    }

    path
}
