use super::*;

#[test]
fn scale_is_dpi_over_96() {
    assert_eq!(scale_from_dpi(Some(96)), 1.0);
    assert_eq!(scale_from_dpi(Some(144)), 1.5);
}

#[test]
fn scale_is_one_when_dpi_is_unknown() {
    assert_eq!(scale_from_dpi(None), 1.0);
    assert_eq!(scale_from_dpi(Some(0)), 1.0);
}

#[test]
fn listed_monitors_have_consistent_geometry() {
    // A session without a desktop may list nothing; that is fine.
    for display in enumerate_monitors().unwrap_or_default() {
        let (b, w) = (display.bounds, display.work_area);
        assert!(b.x <= w.x && b.y <= w.y, "{display:?}");
        assert!(b.x + b.width >= w.x + w.width, "{display:?}");
        assert!(b.y + b.height >= w.y + w.height, "{display:?}");
        assert!(display.scale > 0.0, "{display:?}");
    }
}
