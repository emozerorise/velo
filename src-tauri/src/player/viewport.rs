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
