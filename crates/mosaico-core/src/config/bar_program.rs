//! The bars other programs draw that mosaico knows by name.

/// A bar another program draws that `[bar] provider` can name, and how
/// to tell it runs and speaks for that provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarProgram {
    /// Its name for people, such as `"YASB"`.
    pub title: &'static str,
    /// The program that draws it, by its file name without the folder or
    /// the extension the platform gives programs.
    pub program: &'static str,
    /// The `bar` its sessions give in `hello`, which is also its
    /// provider's name in `[bar] provider`.
    pub bar: &'static str,
}

/// YASB, a bar for Windows written with Qt.
pub const YASB: BarProgram = BarProgram {
    title: "YASB",
    program: "yasb",
    bar: "yasb",
};
