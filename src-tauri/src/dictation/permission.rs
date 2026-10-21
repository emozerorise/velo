use super::types::{DictationError, Result};
use crate::jobs::JobCancel;
use std::{future::Future, pin::Pin, sync::atomic::Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicPermission {
    NotDetermined,
    Granted,
    Denied,
    Restricted,
    Unavailable,
}

pub trait PermissionBackend: Send + Sync {
    fn request(
        &self,
        cancel: JobCancel,
    ) -> Pin<Box<dyn Future<Output = Result<MicPermission>> + Send>>;
}

#[derive(Default)]
pub struct NativePermission;
impl PermissionBackend for NativePermission {
    fn request(
        &self,
        cancel: JobCancel,
    ) -> Pin<Box<dyn Future<Output = Result<MicPermission>> + Send>> {
        Box::pin(request_microphone_permission(cancel))
    }
}

/// Called only by an explicitly started job, never from startup or status reads.
pub async fn request_microphone_permission(cancel: JobCancel) -> Result<MicPermission> {
    if cancel.flag().load(Ordering::Acquire) {
        return Err(DictationError::Cancelled);
    }
    #[cfg(target_os = "macos")]
    {
        let (status, receiver) = macos_request()?;
        if let Some(receiver) = receiver {
            tokio::select! { biased;
                _ = cancel.cancelled() => Err(DictationError::Cancelled),
                result = receiver => result.map(|granted| if granted { MicPermission::Granted } else { MicPermission::Denied }).map_err(|_| DictationError::DeviceUnavailable),
            }
        } else {
            Ok(status)
        }
    }
    #[cfg(target_os = "windows")]
    {
        Ok(MicPermission::Granted)
    } // WASAPI reports access denial when input opens.
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Ok(MicPermission::Unavailable)
    }
}

#[cfg(target_os = "macos")]
fn macos_request() -> Result<(MicPermission, Option<tokio::sync::oneshot::Receiver<bool>>)> {
    use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio};
    use std::sync::Arc;
    // Audio is the only valid media type passed to either authorization API.
    unsafe {
        let audio = AVMediaTypeAudio.ok_or(DictationError::DeviceUnavailable)?;
        let status = match AVCaptureDevice::authorizationStatusForMediaType(audio) {
            AVAuthorizationStatus::Authorized => MicPermission::Granted,
            AVAuthorizationStatus::Denied => MicPermission::Denied,
            AVAuthorizationStatus::Restricted => MicPermission::Restricted,
            AVAuthorizationStatus::NotDetermined => MicPermission::NotDetermined,
            _ => MicPermission::Unavailable,
        };
        if status != MicPermission::NotDetermined {
            return Ok((status, None));
        }
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let sender = Arc::new(parking_lot::Mutex::new(Some(sender)));
        let completion = block2::RcBlock::new(move |granted: objc2::runtime::Bool| {
            if let Some(sender) = sender.lock().take() {
                let _ = sender.send(granted.as_bool());
            }
        });
        AVCaptureDevice::requestAccessForMediaType_completionHandler(audio, &completion);
        Ok((status, Some(receiver)))
    }
}
