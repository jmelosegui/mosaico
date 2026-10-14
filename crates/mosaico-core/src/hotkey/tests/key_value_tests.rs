use crate::hotkey::{Digit, FunctionKey, Letter};

#[test]
fn a_letter_is_kept_in_uppercase() {
    // Act
    let letter = Letter::new('j');

    // Assert
    assert_eq!(letter.map(Letter::get), Some('J'));
    assert_eq!(letter, Letter::new('J'));
}

#[test]
fn only_a_to_z_are_letters() {
    for other in ['1', '!', ' ', '\u{e9}', '\u{3a3}'] {
        // Act
        let letter = Letter::new(other);

        // Assert
        assert_eq!(letter, None, "{other:?}");
    }
}

#[test]
fn digits_go_from_0_to_9() {
    // Assert
    assert_eq!(Digit::new(0).map(Digit::get), Some(0));
    assert_eq!(Digit::new(9).map(Digit::get), Some(9));
    assert_eq!(Digit::new(10), None);
}

#[test]
fn function_keys_go_from_1_to_the_last_one() {
    // Assert
    assert_eq!(FunctionKey::new(0), None);
    assert_eq!(FunctionKey::new(1).map(FunctionKey::get), Some(1));
    let last = FunctionKey::new(FunctionKey::LAST).map(FunctionKey::get);
    assert_eq!(last, Some(FunctionKey::LAST));
    assert_eq!(FunctionKey::new(FunctionKey::LAST + 1), None);
}
