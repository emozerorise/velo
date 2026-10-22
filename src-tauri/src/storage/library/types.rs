use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LibraryData {
    pub version: u32,
    pub revision: u64,
    pub projects: Vec<LibraryProject>,
    pub recordings: Vec<LibraryRecording>,
    pub preferences: LibraryPreferences,
}
impl Default for LibraryData {
    fn default() -> Self {
        Self {
            version: 1,
            revision: 0,
            projects: vec![],
            recordings: vec![],
            preferences: LibraryPreferences::default(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LibraryProject {
    pub id: String,
    pub name: String,
    pub created_at: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LibraryRecording {
    pub id: String,
    pub path: String,
    pub display_name: Option<String>,
    pub project_id: Option<String>,
    pub pinned: bool,
    pub added_at: u64,
    pub duration: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LibrarySort {
    #[default]
    Added,
    Played,
    Name,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LibraryPreferences {
    pub collapsed_project_ids: Vec<String>,
    pub sort: LibrarySort,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub id: String,
    pub path: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LibraryMutation {
    AddRecordings {
        entries: Vec<LibraryEntry>,
        project_id: Option<String>,
    },
    CreateProject {
        id: String,
        name: String,
    },
    RenameProject {
        id: String,
        name: String,
    },
    DeleteProject {
        id: String,
    },
    RenameRecording {
        id: String,
        name: String,
    },
    ResetName {
        id: String,
    },
    MoveRecording {
        id: String,
        project_id: Option<String>,
    },
    SetPin {
        id: String,
        pinned: bool,
    },
    RemoveRecording {
        id: String,
    },
    SetPreferences {
        preferences: LibraryPreferences,
    },
    SetDuration {
        id: String,
        duration: f64,
    },
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LibraryError {
    pub kind: String,
    pub message: String,
}
impl LibraryError {
    pub fn new(kind: &str, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
        }
    }
}
pub type LibraryResult<T> = std::result::Result<T, LibraryError>;
