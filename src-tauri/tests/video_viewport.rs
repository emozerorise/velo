use velo_lib::player::viewport::{
    appkit_rect, win32_rect, ClientMetrics, PixelRect, Rect, ResolvedViewport, SurfaceLayout,
    VideoViewportInput, ViewportState,
};

fn client() -> ClientMetrics {
    ClientMetrics {
        width: 1200.0,
        height: 600.0,
        scale_factor: 2.0,
    }
}
fn input(revision: u64) -> VideoViewportInput {
    VideoViewportInput {
        rect: Rect {
            x: 100.0,
            y: 50.0,
            width: 400.0,
            height: 200.0,
        },
        css_width: 1000.0,
        css_height: 500.0,
        visible: true,
        revision,
    }
}

#[test]
fn converts_css_zoom_to_client_logical_units() {
    let resolved = ViewportState::default()
        .apply(input(1), client())
        .unwrap()
        .unwrap();
    assert_eq!(
        resolved.rect,
        Rect {
            x: 120.0,
            y: 60.0,
            width: 480.0,
            height: 240.0
        }
    );
    assert!(resolved.visible);
}

#[test]
fn rejects_nonfinite_or_nonpositive_metrics() {
    let mut state = ViewportState::default();
    for invalid in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
        let mut geometry = input(1);
        geometry.css_width = invalid;
        assert!(state.apply(geometry, client()).is_err());
        let mut metrics = client();
        metrics.scale_factor = invalid;
        assert!(state.apply(input(1), metrics).is_err());
        let mut geometry = input(1);
        geometry.rect.height = invalid;
        assert!(state.apply(geometry, client()).is_err());
    }
    for invalid in [f64::NAN, f64::NEG_INFINITY, f64::INFINITY] {
        let mut geometry = input(1);
        geometry.rect.x = invalid;
        assert!(state.apply(geometry, client()).is_err());
    }
    assert!(state.apply(input(0), client()).is_err());
    assert!(state.apply(input(9_007_199_254_740_992), client()).is_err());
    assert!(
        state.apply(input(1), client()).unwrap().is_some(),
        "invalid input must not advance revision"
    );
}

#[test]
fn clamps_negative_edges_and_oversize_rect() {
    let mut geometry = input(1);
    geometry.rect = Rect {
        x: -100.0,
        y: -30.0,
        width: 1400.0,
        height: 800.0,
    };
    let result = ViewportState::default()
        .apply(geometry, client())
        .unwrap()
        .unwrap();
    assert_eq!(
        result.rect,
        Rect {
            x: 0.0,
            y: 0.0,
            width: 1200.0,
            height: 600.0
        }
    );
    geometry.revision = 2;
    geometry.rect.x = 2000.0;
    let outside = ViewportState::default()
        .apply(geometry, client())
        .unwrap()
        .unwrap();
    assert!(!outside.visible);
    assert_eq!(outside.rect.width, 0.0);
}

#[test]
fn ignores_old_revision_after_close() {
    let mut state = ViewportState::default();
    state.apply(input(3), client()).unwrap();
    let mut closed = input(5);
    closed.visible = false;
    assert!(!state.apply(closed, client()).unwrap().unwrap().visible);
    assert!(state.apply(input(4), client()).unwrap().is_none());
    assert!(state.apply(input(5), client()).unwrap().is_none());
    assert!(state.apply(input(6), client()).unwrap().unwrap().visible);
}

#[test]
fn appkit_rect_flips_y_once() {
    let rect = Rect {
        x: 100.0,
        y: 50.0,
        width: 300.0,
        height: 200.0,
    };
    assert_eq!(appkit_rect(rect, 600.0), Rect { y: 350.0, ..rect });
}

#[test]
fn win32_rect_scales_and_rounds_edges() {
    let rect = Rect {
        x: 100.0,
        y: 50.0,
        width: 300.0,
        height: 200.0,
    };
    assert_eq!(
        win32_rect(rect, 2.0),
        PixelRect {
            x: 200,
            y: 100,
            width: 600,
            height: 400
        }
    );
    assert_eq!(
        win32_rect(
            Rect {
                x: 0.3,
                y: 0.3,
                width: 100.3,
                height: 50.3
            },
            1.5
        ),
        PixelRect {
            x: 0,
            y: 0,
            width: 151,
            height: 76
        }
    );
}

#[test]
fn hidden_surface_retains_nonzero_live_render_target() {
    let mut surface = SurfaceLayout::new();
    let metrics = ClientMetrics {
        width: 800.0,
        height: 600.0,
        scale_factor: 2.0,
    };
    let opened = ResolvedViewport {
        rect: Rect {
            x: 100.0,
            y: 50.0,
            width: 300.0,
            height: 200.0,
        },
        visible: true,
        revision: 4,
    };
    assert_eq!(surface.apply(opened, metrics), Some(opened.rect));
    let hidden = ResolvedViewport {
        rect: Rect {
            width: 0.0,
            height: 0.0,
            ..opened.rect
        },
        visible: false,
        revision: 6,
    };
    assert_eq!(surface.apply(hidden, metrics), Some(opened.rect));
    assert!(
        surface
            .apply(
                ResolvedViewport {
                    revision: 5,
                    ..opened
                },
                metrics
            )
            .is_none(),
        "late UI work cannot reopen a closed shelf"
    );
    let resized = surface.resize(ClientMetrics {
        width: 80.0,
        height: 40.0,
        ..metrics
    });
    assert!(resized.width > 0.0 && resized.height > 0.0);
    assert!(resized.x + resized.width <= 80.0 && resized.y + resized.height <= 40.0);
}
