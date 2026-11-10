use std::path::Path;

use crate::Result;

use super::{ProcessControl, RunningProcess};

/// A process list and nothing else.
struct Listed(Vec<(u32, &'static str)>);

impl ProcessControl for Listed {
    fn launch_daemon(&self) -> Result<u32> {
        unreachable!("only the list is read")
    }

    fn launch(&self, _: &Path) -> Result<u32> {
        unreachable!("only the list is read")
    }

    fn is_alive(&self, _: u32) -> bool {
        unreachable!("only the list is read")
    }

    fn kill(&self, _: u32) -> Result<()> {
        unreachable!("only the list is read")
    }

    fn processes(&self) -> Vec<RunningProcess> {
        let each = self.0.iter().map(|&(pid, program)| RunningProcess {
            pid,
            program: program.to_string(),
        });
        each.collect()
    }
}

#[test]
fn a_program_is_found_by_its_name_whatever_the_case() {
    // Arrange
    let listed = Listed(vec![
        (4, "YASB"),
        (5, "yasbc"),
        (6, "yasb"),
        (7, "explorer"),
    ]);

    // Act
    let running = listed.running("yasb");

    // Assert
    assert_eq!(running, [4, 6]);
}
