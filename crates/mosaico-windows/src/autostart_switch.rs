//! The Startup apps switch of a Run entry.
//!
//! Settings and Task Manager keep it as a binary value named like the
//! Run value, under `...\Explorer\StartupApproved\Run`. Its first byte
//! holds flags, and the lowest one is set once the user turned the entry
//! off: Windows writes 2 or 6 while the entry may run, and 3 or 7,
//! followed by the time that happened, once it is off. Other programs
//! write other values, such as all zeros, which have that flag clear.
//! The format is not documented; this follows what Windows writes.

/// Whether the switch `data` says the user turned the entry off.
pub fn is_off(data: &[u8]) -> bool {
    data.first().is_some_and(|flags| flags & 1 == 1)
}

#[cfg(test)]
mod tests {
    use super::is_off;

    /// Switch data that starts with `flags`, followed by a time.
    fn switch(flags: u8) -> Vec<u8> {
        let mut data = vec![flags, 0, 0, 0];
        data.extend([0x10, 0x5E, 0x2D, 0x6A, 0x3B, 0x4C, 0xDB, 0x01]);
        data
    }

    #[test]
    fn a_switch_windows_writes_while_the_entry_may_run_is_on() {
        for flags in [2, 6] {
            // Act
            let off = is_off(&switch(flags));

            // Assert
            assert!(!off, "{flags}");
        }
    }

    #[test]
    fn a_switch_windows_writes_once_the_user_turned_it_off_is_off() {
        for flags in [3, 7] {
            // Act
            let off = is_off(&switch(flags));

            // Assert
            assert!(off, "{flags}");
        }
    }

    #[test]
    fn a_switch_without_the_off_flag_is_on() {
        // Arrange: zeros, as some installers write, and an empty value.
        let values: [&[u8]; 2] = [&[0; 12], &[]];

        for data in values {
            // Act
            let off = is_off(data);

            // Assert
            assert!(!off, "{data:?}");
        }
    }
}
