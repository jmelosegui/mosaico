/// A key that a hotkey can be bound to.
///
/// Parsed from the key name of a binding in `keybindings.toml`, ignoring
/// ASCII case, and shown with a name that parses back to the same key.
/// Punctuation keys are named after what they type on a US layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A letter key.
    Letter(Letter),
    /// A digit key above the letters, not on the number pad.
    Digit(Digit),
    /// A function key.
    Function(FunctionKey),
    /// Enter, also written `Return`.
    Enter,
    /// Tab.
    Tab,
    /// Escape, also written `Esc`.
    Escape,
    /// The space bar.
    Space,
    /// Backspace.
    Backspace,
    /// Delete, also written `Del`.
    Delete,
    /// Insert, also written `Ins`.
    Insert,
    /// Home.
    Home,
    /// End.
    End,
    /// Page Up, also written `PgUp`.
    PageUp,
    /// Page Down, also written `PgDn`.
    PageDown,
    /// The left arrow.
    Left,
    /// The up arrow.
    Up,
    /// The right arrow.
    Right,
    /// The down arrow.
    Down,
    /// The `-` key.
    Minus,
    /// The `=` and `+` key, also written `Equals`.
    Plus,
    /// The `,` key.
    Comma,
    /// The `.` key, also written `Dot`.
    Period,
    /// The `/` key.
    Slash,
    /// The `;` key.
    Semicolon,
    /// The `\` key.
    Backslash,
    /// The `[` key.
    LBracket,
    /// The `]` key.
    RBracket,
    /// The `'` key.
    Quote,
    /// The `` ` `` key, also written `Grave`.
    Backtick,
}

/// One of the letters `A` to `Z`, kept in uppercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Letter(char);

impl Letter {
    /// The letter `letter`, in either case, or `None` for anything other
    /// than `A` to `Z`.
    pub fn new(letter: char) -> Option<Self> {
        letter
            .is_ascii_alphabetic()
            .then(|| Self(letter.to_ascii_uppercase()))
    }

    /// The letter, in uppercase.
    pub fn get(self) -> char {
        self.0
    }
}

/// One of the digits `0` to `9`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Digit(u8);

impl Digit {
    /// The digit `digit`, or `None` when it is more than 9.
    pub fn new(digit: u8) -> Option<Self> {
        (digit <= 9).then_some(Self(digit))
    }

    /// The digit.
    pub fn get(self) -> u8 {
        self.0
    }
}

/// The number of a function key, from `1` for `F1` to [`LAST`](Self::LAST).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionKey(u8);

impl FunctionKey {
    /// The number of the last function key a binding can name, `F24`.
    pub const LAST: u8 = 24;

    /// Function key `F<number>`, or `None` when there is no such key.
    pub fn new(number: u8) -> Option<Self> {
        (1..=Self::LAST).contains(&number).then_some(Self(number))
    }

    /// The number of the key, `1` for `F1`.
    pub fn get(self) -> u8 {
        self.0
    }
}

/// The key for the letter `letter`, in tests.
#[cfg(test)]
pub(crate) fn letter(letter: char) -> Key {
    Key::Letter(Letter::new(letter).expect("a letter from A to Z"))
}

/// The key for the digit `digit`, in tests.
#[cfg(test)]
pub(crate) fn digit(digit: u8) -> Key {
    Key::Digit(Digit::new(digit).expect("a digit from 0 to 9"))
}

/// Function key `F<number>`, in tests.
#[cfg(test)]
pub(crate) fn function(number: u8) -> Key {
    Key::Function(FunctionKey::new(number).expect("a function key"))
}

#[cfg(test)]
#[path = "tests/key_value_tests.rs"]
mod tests;
