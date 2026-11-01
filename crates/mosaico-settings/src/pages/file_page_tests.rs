use std::cell::RefCell;
use std::rc::Rc;

use super::*;

/// A page that records what the window asked of it.
#[derive(Default)]
struct Fake {
    page: Option<Page>,
    changes: usize,
    fails: bool,
    calls: Rc<RefCell<Vec<&'static str>>>,
}

impl FilePage for Fake {
    fn page(&self) -> Page {
        self.page.unwrap_or(Page::Bar)
    }

    fn changes(&self) -> usize {
        self.changes
    }

    fn problem(&self) -> Option<&str> {
        None
    }

    fn show(&mut self, _ui: &mut Ui, _c: &Colors, _query: &str) {}

    fn apply(&mut self) -> Result<(), String> {
        self.calls.borrow_mut().push("apply");
        if self.fails {
            return Err("no room".into());
        }
        self.changes = 0;
        Ok(())
    }

    fn revert(&mut self) {
        self.changes = 0;
    }

    fn reload(&mut self) {
        self.calls.borrow_mut().push("reload");
    }
}

#[test]
fn only_pages_with_edits_are_saved_and_the_first_failure_stops_apply() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let page = |changes, fails| -> Box<dyn FilePage> {
        Box::new(Fake {
            changes,
            fails,
            calls: calls.clone(),
            ..Fake::default()
        })
    };
    let mut pages = FilePages::new(vec![page(0, false), page(1, true), page(2, false)]);

    let result = pages.apply();

    assert_eq!(result, Err("no room".into()));
    assert_eq!(*calls.borrow(), ["apply"]);
    assert_eq!(pages.changes(), 3);
}

#[test]
fn only_pages_without_edits_follow_their_files() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let page = |changes| -> Box<dyn FilePage> {
        Box::new(Fake {
            changes,
            calls: calls.clone(),
            ..Fake::default()
        })
    };
    let mut pages = FilePages::new(vec![page(0), page(1)]);

    pages.reload_unedited();

    assert_eq!(*calls.borrow(), ["reload"]);
}

#[test]
fn a_page_is_found_by_the_navigation_entry_it_draws() {
    let rules = Fake {
        page: Some(Page::Rules),
        changes: 4,
        ..Fake::default()
    };
    let mut pages = FilePages::new(vec![Box::new(Fake::default()), Box::new(rules)]);

    assert_eq!(pages.get(Page::Rules).map(|p| p.changes()), Some(4));
    assert!(pages.get(Page::Keybindings).is_none());
}
