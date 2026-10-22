//! Explicit private artifacts for opted-in acceptance; no JobInput/headers/keys.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Mutex,
    time::Instant,
};
use velo_lib::{
    errors::{Result, VeloError},
    summary::pipeline::EvidenceObserver,
};
pub struct Artifacts {
    dir: PathBuf,
    start: Instant,
    state: Mutex<State>,
}

pub fn application_labels(markdown: &str) -> Vec<&str> {
    markdown
        .char_indices()
        .filter_map(|(at, ch)| {
            if ch != '['
                || markdown.as_bytes()[..at]
                    .iter()
                    .rev()
                    .take_while(|&&b| b == b'\\')
                    .count()
                    % 2
                    != 0
            {
                return None;
            }
            markdown[at + 1..].split_once(']').map(|(label, _)| label)
        })
        .collect()
}
pub fn is_plain_paragraph(markdown: &str) -> bool {
    let mut lines = markdown
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let Some(line) = lines.next() else {
        return false;
    };
    lines.next().is_none()
        && !line.starts_with(['#', '-', '*', '+'])
        && !line.split_once(". ").is_some_and(|(prefix, _)| {
            !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_digit())
        })
}
#[derive(Default)]
struct State {
    index: usize,
    requests: usize,
    packets: BTreeSet<String>,
    batches: BTreeSet<String>,
    stages: BTreeMap<String, usize>,
}
impl Artifacts {
    pub fn new(dir: PathBuf) -> Self {
        std::fs::create_dir(&dir).expect("new private artifact directory required");
        Self {
            dir,
            start: Instant::now(),
            state: Mutex::new(State::default()),
        }
    }
    pub fn manifest(&self) -> serde_json::Value {
        let state = self.state.lock().unwrap();
        serde_json::json!({"requests":state.requests,"extraction_packets":state.packets.len(),"verification_batches":state.batches.len(),"ceiling":2*state.packets.len()+2*state.batches.len()+2,"stages":state.stages,"elapsed_ms":self.start.elapsed().as_millis()})
    }
}
impl EvidenceObserver for Artifacts {
    fn record(&self, event: &str, payload: &serde_json::Value) -> Result<()> {
        let fields: &[&str] = match event {
            "request" => &["stage", "system", "user", "max_tokens", "context_tokens"],
            "response" => &["stage", "text"],
            "points" => &["points"],
            "plan" => &["layout", "occurrences"],
            "review" => &["job", "checks", "need_sources"],
            "composed" => &["markdown"],
            _ => return Err(VeloError::Summary("Unknown private artifact event".into())),
        };
        if !payload.as_object().is_some_and(|object| {
            object.len() == fields.len() && object.keys().all(|key| fields.contains(&key.as_str()))
        }) {
            return Err(VeloError::Summary(
                "Nonallowlisted private artifact metadata".into(),
            ));
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| VeloError::Summary("Private artifact lock failed".into()))?;
        state.index += 1;
        if event == "request" {
            state.requests += 1;
            let stage = payload["stage"].as_str().unwrap_or("unknown");
            *state.stages.entry(stage.into()).or_default() += 1;
            if let Some(user) = payload["user"]
                .as_str()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            {
                if stage == "extracting" {
                    let ids: BTreeSet<_> = user["sources"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|row| row["source"].as_str())
                        .collect();
                    state.packets.insert(serde_json::to_string(&ids).unwrap());
                }
                if stage == "checking" {
                    let ids: BTreeSet<_> = user["occurrences"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|row| row["id"].as_str())
                        .collect();
                    state.batches.insert(serde_json::to_string(&ids).unwrap());
                }
            }
        }
        let bytes=serde_json::to_vec_pretty(&serde_json::json!({"event":event,"elapsed_ms":self.start.elapsed().as_millis(),"payload":payload})).map_err(|_|VeloError::Summary("Could not encode private artifact".into()))?;
        std::fs::write(
            self.dir.join(format!("{:03}-{event}.json", state.index)),
            bytes,
        )
        .map_err(|_| VeloError::Summary("Could not save private acceptance artifact".into()))
    }
}
