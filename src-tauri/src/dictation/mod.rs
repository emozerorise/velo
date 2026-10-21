//! Short local microphone drafts. Feature orchestration belongs in commands/.
pub mod audio;
pub mod capture;
pub mod permission;
pub mod types;

use crate::jobs::{JobCancel, JobGate, JobResource};
use capture::CaptureBackend;
use parking_lot::Mutex;
use permission::{MicPermission, PermissionBackend};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
};
use types::*;

pub type EngineFn =
    Arc<dyn Fn(&Path, &[f32], &str, &str, &JobCancel) -> Result<String> + Send + Sync>;
pub type EventSink = Arc<dyn Fn(DictationEvent) + Send + Sync>;
pub type PausePlayer = Arc<dyn Fn() -> Result<()> + Send + Sync>;
pub type StillAllowed = Arc<dyn Fn() -> bool + Send + Sync>;

pub struct DictationInput {
    pub path: String,
    pub session_id: String,
    pub model: PathBuf,
    pub language: String,
    pub vocabulary: String,
    pub engine: EngineFn,
}

#[derive(Default, Clone)]
pub struct DictationState {
    active: Arc<Mutex<Option<Active>>>,
}
struct Active {
    id: String,
    token: Arc<()>,
    cancel: JobCancel,
    stop: Arc<AtomicBool>,
    phase: Arc<AtomicU8>,
}
struct Cleanup {
    state: DictationState,
    token: Arc<()>,
}
pub struct DictationReservation {
    id: String,
    cleanup: Cleanup,
    cancel: JobCancel,
    stop: Arc<AtomicBool>,
    phase: Arc<AtomicU8>,
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        let mut active = self.state.active.lock();
        if active
            .as_ref()
            .is_some_and(|a| Arc::ptr_eq(&a.token, &self.token))
        {
            *active = None;
        }
    }
}

impl DictationState {
    pub fn reserve(&self, id: &str) -> Result<DictationReservation> {
        validate_id(id)?;
        let mut active = self.active.lock();
        if active.is_some() {
            return Err(DictationError::EngineBusy);
        }
        let token = Arc::new(());
        let cancel = JobCancel::new();
        let stop = Arc::new(AtomicBool::new(false));
        let phase = Arc::new(AtomicU8::new(0));
        *active = Some(Active {
            id: id.into(),
            token: token.clone(),
            cancel: cancel.clone(),
            stop: stop.clone(),
            phase: phase.clone(),
        });
        Ok(DictationReservation {
            id: id.into(),
            cleanup: Cleanup {
                state: self.clone(),
                token,
            },
            cancel,
            stop,
            phase,
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &self,
        input: DictationInput,
        permission: Arc<dyn PermissionBackend>,
        capture: Arc<dyn CaptureBackend>,
        pause: PausePlayer,
        allowed: StillAllowed,
        jobs: &JobGate,
        emit: EventSink,
    ) -> Result<()> {
        self.reserve(&input.session_id)?
            .start(input, permission, capture, pause, allowed, jobs, emit)
    }
    pub fn stop(&self, id: &str) -> Result<()> {
        validate_id(id)?;
        if let Some(active) = self.active.lock().as_ref().filter(|a| a.id == id) {
            active.stop.store(true, Ordering::Release);
            if active.phase.load(Ordering::Acquire) == 0 {
                active.cancel.cancel();
            }
        }
        Ok(())
    }
    pub fn cancel(&self, id: &str) -> Result<()> {
        validate_id(id)?;
        if let Some(active) = self.active.lock().as_ref().filter(|a| a.id == id) {
            active.cancel.cancel();
        }
        Ok(())
    }
    pub fn cancel_all(&self) {
        if let Some(active) = self.active.lock().as_ref() {
            active.cancel.cancel();
        }
    }
    pub fn cancel_on_blur(&self) {
        if let Some(active) = self.active.lock().as_ref() {
            if active.phase.load(Ordering::Acquire) != 0 {
                active.cancel.cancel();
            }
        }
    }
}

impl DictationReservation {
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        self,
        mut input: DictationInput,
        permission: Arc<dyn PermissionBackend>,
        capture: Arc<dyn CaptureBackend>,
        pause: PausePlayer,
        allowed: StillAllowed,
        jobs: &JobGate,
        emit: EventSink,
    ) -> Result<()> {
        if self.id != input.session_id || input.language.len() > 64 || input.vocabulary.len() > 8192
        {
            return Err(DictationError::InvalidInput);
        }
        let path = std::fs::canonicalize(&input.path).map_err(|_| DictationError::InvalidInput)?;
        if !path.is_file() {
            return Err(DictationError::InvalidInput);
        }
        input.path = path.to_str().ok_or(DictationError::InvalidInput)?.into();
        if !input.model.is_file() {
            return Err(DictationError::MissingModel);
        }
        let lease = jobs
            .try_acquire(JobResource::Whisper, &input.session_id)
            .map_err(|_| DictationError::EngineBusy)?;
        if self.cancel.flag().load(Ordering::Acquire) {
            return Err(DictationError::Cancelled);
        }
        tokio::spawn(async move {
            let _cleanup = self.cleanup;
            let path = input.path.clone();
            let id = input.session_id.clone();
            emit(DictationEvent::State(DictationProgress {
                path: path.clone(),
                session_id: id.clone(),
                phase: DictationPhase::RequestingPermission,
                elapsed_ms: 0,
                paused_playback: false,
            }));
            let granted = tokio::select! { biased;
                _ = self.cancel.cancelled() => Err(DictationError::Cancelled),
                result = permission.request(self.cancel.clone()) => result.and_then(|status| match status {
                    MicPermission::Granted => Ok(()), MicPermission::Denied => Err(DictationError::PermissionDenied),
                    MicPermission::Restricted => Err(DictationError::PermissionRestricted),
                    _ => Err(DictationError::DeviceUnavailable),
                }),
            };
            let cancelled = self.cancel.clone();
            let result = match granted {
                Err(error) => Err(error),
                Ok(()) => {
                    let progress = emit.clone();
                    tokio::task::spawn_blocking(move || {
                        let _lease = lease;
                        let check = || {
                            if self.cancel.flag().load(Ordering::Acquire) || !allowed() {
                                Err(DictationError::Cancelled)
                            } else {
                                Ok(())
                            }
                        };
                        check()?;
                        self.phase.store(1, Ordering::Release);
                        check()?;
                        pause()?;
                        check()?;
                        let audio = capture::record_until_stopped(
                            capture.as_ref(),
                            &self.cancel,
                            &self.stop,
                            |elapsed_ms| {
                                progress(DictationEvent::State(DictationProgress {
                                    path: input.path.clone(),
                                    session_id: input.session_id.clone(),
                                    phase: DictationPhase::Recording,
                                    elapsed_ms,
                                    paused_playback: true,
                                }))
                            },
                        )?;
                        check()?;
                        self.phase.store(2, Ordering::Release);
                        progress(DictationEvent::State(DictationProgress {
                            path: input.path.clone(),
                            session_id: input.session_id.clone(),
                            phase: DictationPhase::Transcribing,
                            elapsed_ms: 0,
                            paused_playback: true,
                        }));
                        let samples = audio::prepare_audio(audio)?;
                        check()?;
                        let text = (input.engine)(
                            &input.model,
                            &samples,
                            &input.language,
                            &input.vocabulary,
                            &self.cancel,
                        )?;
                        check()?;
                        let text = text.trim();
                        if text.is_empty() || text.len() > 65536 {
                            return Err(DictationError::TranscriptionFailed);
                        }
                        Ok(text.to_string())
                    })
                    .await
                    .unwrap_or(Err(DictationError::TranscriptionFailed))
                }
            };
            let result = if cancelled.flag().load(Ordering::Acquire) {
                Err(DictationError::Cancelled)
            } else {
                result
            };
            match result {
                Ok(text) => emit(DictationEvent::Ready(DictationReady {
                    path,
                    session_id: id,
                    text,
                })),
                Err(code) => emit(DictationEvent::Error(DictationFailure {
                    path,
                    session_id: id,
                    code,
                    cancelled: code == DictationError::Cancelled,
                })),
            }
        });
        Ok(())
    }
}
