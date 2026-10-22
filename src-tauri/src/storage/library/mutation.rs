use super::types::*;
use std::{
    collections::HashSet,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

fn invalid(message: &str) -> LibraryError {
    LibraryError::new("invalid", message)
}
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && id.is_ascii() && id != "uncategorized"
}
fn name(value: &str) -> LibraryResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 120 {
        return Err(invalid("Name must contain 1–120 characters"));
    }
    Ok(value.into())
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_millis() as u64)
        .unwrap_or(0)
}
pub fn validate(data: &LibraryData) -> LibraryResult<()> {
    if data.version != 1 {
        return Err(LibraryError::new(
            "unsupported_version",
            "Unsupported library version",
        ));
    }
    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    for p in &data.projects {
        if !valid_id(&p.id)
            || !ids.insert(p.id.as_str())
            || name(&p.name)? != p.name
            || !names.insert(p.name.to_lowercase())
        {
            return Err(invalid("Invalid or duplicate project"));
        }
    }
    let projects = ids.clone();
    let mut paths = HashSet::new();
    for r in &data.recordings {
        if !valid_id(&r.id)
            || !ids.insert(&r.id)
            || !Path::new(&r.path).is_absolute()
            || r.path.contains('\0')
            || !paths.insert(&r.path)
        {
            return Err(invalid("Invalid or duplicate recording"));
        }
        if let Some(n) = &r.display_name {
            if name(n)? != *n {
                return Err(invalid("Invalid display name"));
            }
        }
        if r.project_id
            .as_ref()
            .is_some_and(|id| !projects.contains(id.as_str()))
        {
            return Err(invalid("Unknown project"));
        }
        if r.duration.is_some_and(|v| !v.is_finite() || v < 0.0) {
            return Err(invalid("Invalid duration"));
        }
    }
    let mut collapsed = HashSet::new();
    for id in &data.preferences.collapsed_project_ids {
        if (id != "uncategorized" && !projects.contains(id.as_str())) || !collapsed.insert(id) {
            return Err(invalid("Invalid collapsed group"));
        }
    }
    Ok(())
}
fn recording<'a>(data: &'a mut LibraryData, id: &str) -> LibraryResult<&'a mut LibraryRecording> {
    data.recordings
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| invalid("Unknown recording"))
}
pub fn apply_mutation(data: &LibraryData, mutation: LibraryMutation) -> LibraryResult<LibraryData> {
    validate(data)?;
    let mut next = data.clone();
    match mutation {
        LibraryMutation::AddRecordings {
            entries,
            project_id,
        } => {
            if project_id
                .as_ref()
                .is_some_and(|id| !next.projects.iter().any(|p| &p.id == id))
            {
                return Err(invalid("Unknown project"));
            }
            for e in entries {
                if !next.recordings.iter().any(|r| r.path == e.path) {
                    next.recordings.push(LibraryRecording {
                        id: e.id,
                        path: e.path,
                        display_name: None,
                        project_id: project_id.clone(),
                        pinned: false,
                        added_at: now(),
                        duration: None,
                    });
                }
            }
        }
        LibraryMutation::CreateProject { id, name: n } => next.projects.push(LibraryProject {
            id,
            name: name(&n)?,
            created_at: now(),
        }),
        LibraryMutation::RenameProject { id, name: n } => {
            next.projects
                .iter_mut()
                .find(|p| p.id == id)
                .ok_or_else(|| invalid("Unknown project"))?
                .name = name(&n)?
        }
        LibraryMutation::DeleteProject { id } => {
            if !next.projects.iter().any(|p| p.id == id) {
                return Err(invalid("Unknown project"));
            }
            next.projects.retain(|p| p.id != id);
            for r in &mut next.recordings {
                if r.project_id.as_deref() == Some(&id) {
                    r.project_id = None;
                }
            }
            next.preferences.collapsed_project_ids.retain(|v| v != &id);
        }
        LibraryMutation::RenameRecording { id, name: n } => {
            recording(&mut next, &id)?.display_name = Some(name(&n)?)
        }
        LibraryMutation::ResetName { id } => recording(&mut next, &id)?.display_name = None,
        LibraryMutation::MoveRecording { id, project_id } => {
            recording(&mut next, &id)?.project_id = project_id
        }
        LibraryMutation::SetPin { id, pinned } => recording(&mut next, &id)?.pinned = pinned,
        LibraryMutation::RemoveRecording { id } => {
            recording(&mut next, &id)?;
            next.recordings.retain(|r| r.id != id);
        }
        LibraryMutation::SetPreferences { preferences } => next.preferences = preferences,
        LibraryMutation::SetDuration { id, duration } => {
            recording(&mut next, &id)?.duration = Some(duration)
        }
    }
    validate(&next)?;
    Ok(next)
}
