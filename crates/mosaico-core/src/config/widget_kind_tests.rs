use super::*;

#[test]
fn each_kind_is_named_as_the_type_of_a_widget_entry() {
    // Act / Assert -- the names stay those `[[left]]` and `[[right]]` read.
    for kind in WidgetKind::ALL {
        let widget: WidgetConfig = toml::from_str(&format!("type = \"{kind}\"")).unwrap();
        assert_eq!(widget.kind(), kind, "{kind}");
        assert_eq!(kind.name().parse::<WidgetKind>(), Ok(kind));
    }
}

#[test]
fn a_name_that_no_widget_has_is_no_kind() {
    // Act
    let names = ["volume", "Layout", "active-window", ""].map(str::parse::<WidgetKind>);

    // Assert
    assert!(names.iter().all(Result::is_err), "{names:?}");
}

#[test]
fn every_kind_is_listed_once() {
    // Act
    let mut names: Vec<&str> = WidgetKind::ALL.iter().map(|kind| kind.name()).collect();
    names.sort_unstable();
    names.dedup();

    // Assert
    assert_eq!(names.len(), WidgetKind::ALL.len());
}
