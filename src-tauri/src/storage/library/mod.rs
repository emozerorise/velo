mod mutation;
mod types;
pub use mutation::{apply_mutation, validate};
pub use types::*;

use std::{fs, path::PathBuf};

pub struct LibraryStore {
    directory: PathBuf,
    loaded: Option<LibraryData>,
}
impl LibraryStore {
    pub fn open(directory: PathBuf) -> Self {
        Self {
            directory,
            loaded: None,
        }
    }
    pub fn load(&mut self) -> LibraryResult<LibraryData> {
        self.loaded = None;
        let data = match fs::read(self.directory.join("library.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| LibraryError::new("storage", e.to_string()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => LibraryData::default(),
            Err(e) => return Err(LibraryError::new("storage", e.to_string())),
        };
        validate(&data)?;
        self.loaded = Some(data.clone());
        Ok(data)
    }
    pub fn mutate(
        &mut self,
        expected_revision: u64,
        mutation: LibraryMutation,
    ) -> LibraryResult<LibraryData> {
        self.commit_with(expected_revision, mutation, |directory, data| {
            fs::create_dir_all(directory)?;
            let temp = directory.join(format!("library-{}.tmp", std::process::id()));
            fs::write(&temp, serde_json::to_vec_pretty(data)?)?;
            fs::rename(temp, directory.join("library.json"))
        })
    }
    fn commit_with(
        &mut self,
        expected_revision: u64,
        mutation: LibraryMutation,
        write: impl FnOnce(&std::path::Path, &LibraryData) -> std::io::Result<()>,
    ) -> LibraryResult<LibraryData> {
        let data = self.loaded.as_ref().ok_or_else(|| {
            LibraryError::new("storage", "Library must load successfully before editing")
        })?;
        if data.revision != expected_revision {
            return Err(LibraryError::new(
                "conflict",
                "Library changed; reload and retry",
            ));
        }
        let mut next = apply_mutation(data, mutation)?;
        next.revision = data
            .revision
            .checked_add(1)
            .filter(|n| *n <= 9_007_199_254_740_991)
            .ok_or_else(|| LibraryError::new("invalid", "Revision overflow"))?;
        write(&self.directory, &next).map_err(|e| LibraryError::new("storage", e.to_string()))?;
        self.loaded = Some(next.clone());
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_writer_keeps_snapshot_and_revision() {
        let original = LibraryData::default();
        let mut store = LibraryStore {
            directory: PathBuf::new(),
            loaded: Some(original.clone()),
        };
        let error = store.commit_with(
            0,
            LibraryMutation::CreateProject {
                id: "p".into(),
                name: "Team".into(),
            },
            |_, _| Err(std::io::Error::other("write failed")),
        );
        assert!(error.is_err());
        assert_eq!(store.loaded, Some(original));
    }
}
