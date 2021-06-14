use anyhow::{anyhow, Context, Result};
use std::{
    ffi, fs,
    io::Read,
    path::{Path, PathBuf},
};

pub struct Resources {
    root_path: PathBuf,
}

impl Resources {
    pub fn from_relative_exe_path(rel_path: &Path) -> Result<Resources> {
        let exe_file_name = ::std::env::current_exe().context("No exe filename")?;
        let exe_path = exe_file_name.parent().context("No exe parent")?;

        Ok(Resources {
            root_path: exe_path.join(rel_path),
        })
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
        let model = models.pop().ok_or(anyhow!("Obj file has no model!"))?;

        Ok(model.mesh)
    }
}

fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.split('/') {
        path = path.join(part);
    }

    path
}
