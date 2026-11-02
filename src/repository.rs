use crate::model::Project;
use fs2::FileExt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct Repository {
    root: PathBuf,
    _lock: File,
}

impl Repository {
    pub fn open(root: impl AsRef<Path>) -> io::Result<Self> {
        fs::create_dir_all(root.as_ref())?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.as_ref().join(".lock"))?;
        lock.try_lock_exclusive().map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("数据目录已被另一个服务占用，或无法加锁：{error}"),
            )
        })?;
        Ok(Self {
            root: root.as_ref().to_path_buf(),
            _lock: lock,
        })
    }

    pub fn list(&self) -> io::Result<Vec<Project>> {
        let mut projects = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
                let id = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .and_then(|stem| Uuid::parse_str(stem).ok());
                if let Some(id) = id {
                    projects.push(self.get(id)?);
                }
            }
        }
        projects.sort_by_key(|project| std::cmp::Reverse(project.updated_at));
        Ok(projects)
    }

    pub fn get(&self, id: Uuid) -> io::Result<Project> {
        let bytes = fs::read(self.root.join(format!("{id}.json")))?;
        let project: Project = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
        if project.schema_version != 1 || project.id != id {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "项目版本或标识不匹配",
            ));
        }
        Ok(project)
    }

    pub fn save(&self, project: &Project) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(project).map_err(io::Error::other)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.root)?;
        temporary.write_all(&bytes)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(self.root.join(format!("{}.json", project.id)))
            .map_err(|error| error.error)?;
        Ok(())
    }

    pub fn delete(&self, id: Uuid) -> io::Result<()> {
        fs::remove_file(self.root.join(format!("{id}.json")))
    }
}
