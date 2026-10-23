//! Private, explicitly opted-in test artifacts; never a production cache.
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};
use velo_lib::{
    errors::{Result, VeloError},
    summary::pipeline::EvidenceObserver,
};

pub struct Artifacts {
    dir: PathBuf,
    next: Mutex<usize>,
}
impl Artifacts {
    pub fn new(dir: PathBuf) -> Result<Self> {
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder
            .create(&dir)
            .map_err(|_| VeloError::Storage("Diagnostic directory must be new".into()))?;
        Ok(Self {
            dir,
            next: Mutex::new(0),
        })
    }
}
impl EvidenceObserver for Artifacts {
    fn record(&self, event: &str, payload: &serde_json::Value) -> Result<()> {
        let fields: &[&str] = match event {
            "consistency_request" | "confirmation_request" => {
                &["round", "window", "rows", "request"]
            }
            "consistency_response" | "confirmation_response" => &["round", "window", "reply"],
            _ => return Err(VeloError::Storage("Unknown diagnostic event".into())),
        };
        let exact = |value: &serde_json::Value, keys: &[&str]| {
            value.as_object().is_some_and(|object| {
                object.len() == keys.len() && object.keys().all(|key| keys.contains(&key.as_str()))
            })
        };
        if !exact(payload, fields)
            || (event.ends_with("_request")
                && (!exact(
                    &payload["request"],
                    &["system", "user", "model", "context_tokens", "max_tokens"],
                ) || !payload["rows"].as_array().is_some_and(|rows| {
                    rows.iter()
                        .all(|row| exact(row, &["id", "start", "end", "text"]))
                })))
        {
            return Err(VeloError::Storage(
                "Nonallowlisted diagnostic fields".into(),
            ));
        }
        let mut next = self
            .next
            .lock()
            .map_err(|_| VeloError::Storage("Diagnostic lock failed".into()))?;
        let path = self.dir.join(format!("{next:04}-{event}.json"));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(path)
            .map_err(|_| VeloError::Storage("Cannot create new diagnostic file".into()))?;
        let bytes = serde_json::to_vec_pretty(payload)
            .map_err(|_| VeloError::Storage("Cannot encode diagnostic".into()))?;
        file.write_all(&bytes)
            .map_err(|_| VeloError::Storage("Cannot write diagnostic".into()))?;
        *next += 1;
        Ok(())
    }
}

#[test]
fn private_sink_rejects_extra_fields_and_existing_directory() {
    let dir = std::env::temp_dir().join(format!(
        "velo-private-diagnostics-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let artifacts = Artifacts::new(dir.clone()).unwrap();
    assert!(Artifacts::new(dir.clone()).is_err());
    assert!(artifacts
        .record(
            "consistency_response",
            &serde_json::json!({"round":2,"window":1,"reply":"{}","api_key":"NEVER_RECORD"})
        )
        .is_err());
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
    artifacts
        .record(
            "consistency_response",
            &serde_json::json!({"round":2,"window":1,"reply":"{}"}),
        )
        .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(dir.join("0000-consistency_response.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
