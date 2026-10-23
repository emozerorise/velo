//! Loading is selection, not permission to start playback.
use super::core::MpvCore;
use crate::errors::Result;

pub(super) fn load_paused(core: &MpvCore, path: &str, start_time: Option<f64>) -> Result<()> {
    // Pause before replacing the file, so neither the old nor the new source
    // can emit a brief audio burst while an asynchronous load completes.
    core.set_property_bool("pause", true)?;
    if let Some(pos) = start_time {
        let start_arg = format!("start={pos}");
        // mpv 0.38 added a playlist index before per-file options. Keep the
        // older form as a fallback for installations with earlier libmpv.
        if core
            .command(&["loadfile", path, "replace", "-1", &start_arg])
            .is_err()
        {
            core.command(&["loadfile", path, "replace", &start_arg])?;
        }
    } else {
        core.command(&["loadfile", path, "replace"])?;
    }
    Ok(())
}
