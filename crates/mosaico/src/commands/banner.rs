/// Prints the mosaico ASCII logo with version, used by start and doctor.
pub fn print_logo() {
    let c = "\x1b[94m"; // Bright blue — logo
    let d = "\x1b[90m"; // Dim gray — version
    let r = "\x1b[0m"; // Reset
    let v = env!("CARGO_PKG_VERSION");

    let ver = format!("v{v}");
    let pad = " ".repeat(36_usize.saturating_sub(ver.len()));

    println!();
    println!("  {c}█▀▄▀█ █▀▀█ █▀▀▀ █▀▀█ ▀█▀ █▀▀▀ █▀▀█{r}");
    println!("  {c}█ ▀ █ █  █ ▀▀▀█ █▀▀█  █  █    █  █{r}");
    println!("  {c}▀   ▀ ▀▀▀▀ ▀▀▀▀ ▀  ▀ ▀▀▀ ▀▀▀▀ ▀▀▀▀{r}");
    println!("{pad}{d}{ver}{r}");
}
