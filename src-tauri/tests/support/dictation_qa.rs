//! Explicit QA-only adapters: no microphone, OS consent, real model or network.
use crate::dictation::{
    capture::{CaptureBackend, CaptureSession},
    permission::{MicPermission, PermissionBackend},
    types::{CapturedAudio, DictationError, Result},
    EngineFn,
};
use crate::jobs::JobCancel;
use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc, time::Duration};

pub struct Adapters {
    pub model: PathBuf,
    pub permission: Arc<dyn PermissionBackend>,
    pub capture: Arc<dyn CaptureBackend>,
    pub engine: EngineFn,
}
pub fn adapters(identifier: &str) -> Result<Adapters> {
    if identifier != "com.velo.summary-controls-qa" {
        return Err(DictationError::InvalidInput);
    }
    Ok(Adapters {
        model: std::env::current_exe().map_err(|_| DictationError::MissingModel)?,
        permission: Arc::new(SyntheticPermission),
        capture: Arc::new(SyntheticCapture),
        engine: Arc::new(|_, _, _, _, cancel| {
            for _ in 0..40 {
                if cancel.flag().load(std::sync::atomic::Ordering::Acquire) {
                    return Err(DictationError::Cancelled);
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok("ข้อความสังเคราะห์สำหรับทดสอบไมค์ กรุณาแยกงานที่ตกลงแล้วกับคำถามที่ต้องยืนยัน".into())
        }),
    })
}
struct SyntheticPermission;
impl PermissionBackend for SyntheticPermission {
    fn request(&self, _: JobCancel) -> Pin<Box<dyn Future<Output = Result<MicPermission>> + Send>> {
        Box::pin(async {
            tokio::time::sleep(Duration::from_millis(150)).await;
            Ok(MicPermission::Granted)
        })
    }
}
struct SyntheticCapture;
impl CaptureBackend for SyntheticCapture {
    fn start(&self, _: JobCancel) -> Result<Box<dyn CaptureSession>> {
        Ok(Box::new(SyntheticCapture))
    }
}
impl CaptureSession for SyntheticCapture {
    fn stop(self: Box<Self>) -> Result<CapturedAudio> {
        Ok(CapturedAudio {
            sample_rate: 16000,
            mono_samples: (0..16000)
                .map(|i| (i as f32 * std::f32::consts::TAU * 440.0 / 16000.0).sin() * 0.1)
                .collect(),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn qa_adapters_refuse_production_and_unknown_identifiers() {
        assert!(adapters("com.velo.player").is_err());
        assert!(adapters("other").is_err());
        assert!(adapters("com.velo.summary-controls-qa").is_ok());
    }
}
