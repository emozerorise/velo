//! Window-scoped geometry: CSS zoom and screen scale are separate conversions.
use crate::errors::{Result, VeloError};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VideoViewportInput {
    pub rect: Rect,
    pub css_width: f64,
    pub css_height: f64,
    pub visible: bool,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct ClientMetrics {
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedViewport {
    pub rect: Rect,
    pub visible: bool,
    pub revision: u64,
}

pub fn appkit_rect(rect: Rect, client_height: f64) -> Rect {
    Rect {
        y: client_height - rect.y - rect.height,
        ..rect
    }
}

#[derive(Debug, PartialEq)]
pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn win32_rect(rect: Rect, scale: f64) -> PixelRect {
    let right = ((rect.x + rect.width) * scale).round().max(1.0) as i32;
    let bottom = ((rect.y + rect.height) * scale).round().max(1.0) as i32;
    let x = ((rect.x * scale).round() as i32).min(right - 1);
    let y = ((rect.y * scale).round() as i32).min(bottom - 1);
    PixelRect {
        x,
        y,
        width: right - x,
        height: bottom - y,
    }
}

/// Hiding a tool covers its last live target rather than starving mpv's
/// draw/swap loop with a zero-sized or destroyed native view.
pub struct SurfaceLayout {
    revision: u64,
    rect: Option<Rect>,
}

impl Default for SurfaceLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceLayout {
    pub const fn new() -> Self {
        Self {
            revision: 0,
            rect: None,
        }
    }

    pub fn apply(&mut self, viewport: ResolvedViewport, client: ClientMetrics) -> Option<Rect> {
        if viewport.revision <= self.revision {
            return None;
        }
        self.revision = viewport.revision;
        if viewport.visible {
            self.rect = Some(viewport.rect);
        }
        Some(self.resize(client))
    }

    pub fn resize(&mut self, client: ClientMetrics) -> Rect {
        let rect = self.rect.unwrap_or(Rect {
            x: 0.0,
            y: 0.0,
            width: client.width,
            height: client.height,
        });
        let width = client.width.max(1.0);
        let height = client.height.max(1.0);
        let x = rect.x.clamp(0.0, width - 1.0);
        let y = rect.y.clamp(0.0, height - 1.0);
        let rect = Rect {
            x,
            y,
            width: rect.width.clamp(1.0, width - x),
            height: rect.height.clamp(1.0, height - y),
        };
        self.rect = Some(rect);
        rect
    }
}

#[derive(Default)]
pub struct ViewportState {
    revision: u64,
}

impl ViewportState {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn apply(
        &mut self,
        input: VideoViewportInput,
        client: ClientMetrics,
    ) -> Result<Option<ResolvedViewport>> {
        let invalid = || VeloError::InvalidParameter("Invalid video viewport geometry".into());
        if input.revision == 0 || input.revision > 9_007_199_254_740_991 {
            return Err(invalid());
        }
        let positive = [
            input.css_width,
            input.css_height,
            input.rect.width,
            input.rect.height,
            client.width,
            client.height,
            client.scale_factor,
        ];
        if positive
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
            || !input.rect.x.is_finite()
            || !input.rect.y.is_finite()
        {
            return Err(invalid());
        }
        let sx = client.width / input.css_width;
        let sy = client.height / input.css_height;
        let edges = [
            input.rect.x * sx,
            input.rect.y * sy,
            (input.rect.x + input.rect.width) * sx,
            (input.rect.y + input.rect.height) * sy,
        ];
        if edges.iter().any(|edge| !edge.is_finite()) {
            return Err(invalid());
        }
        if input.revision <= self.revision {
            return Ok(None);
        }
        let x = edges[0].clamp(0.0, client.width);
        let y = edges[1].clamp(0.0, client.height);
        let width = (edges[2].clamp(0.0, client.width) - x).max(0.0);
        let height = (edges[3].clamp(0.0, client.height) - y).max(0.0);
        self.revision = input.revision;
        Ok(Some(ResolvedViewport {
            rect: Rect {
                x,
                y,
                width,
                height,
            },
            visible: input.visible && width > 0.0 && height > 0.0,
            revision: input.revision,
        }))
    }
}
