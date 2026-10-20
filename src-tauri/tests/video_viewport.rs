use velo_lib::player::viewport::{ClientMetrics, Rect, VideoViewportInput, ViewportState};

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
