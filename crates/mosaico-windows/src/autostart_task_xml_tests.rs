use super::super::command_line::Action;
use super::definition;

const USER: &str = "S-1-5-21-1-2-3-1001";

fn action(program: &str, arguments: &str) -> Action {
    Action {
        program: program.into(),
        arguments: arguments.into(),
    }
}

#[test]
fn the_task_runs_the_action_at_the_users_logon() {
    // Act
    let xml = definition(USER, &action(r"C:\apps\mosaico.exe", "start --quiet"));

    // Assert
    assert!(
        xml.contains(&format!(
            "<LogonTrigger>\n      <Enabled>true</Enabled>\n      <UserId>{USER}</UserId>"
        )),
        "{xml}"
    );
    assert!(
        xml.contains(&format!(
            "<UserId>{USER}</UserId>\n      <LogonType>InteractiveToken</LogonType>"
        )),
        "{xml}"
    );
    assert!(
        xml.contains(r"<Command>C:\apps\mosaico.exe</Command>"),
        "{xml}"
    );
    assert!(
        xml.contains("<Arguments>start --quiet</Arguments>"),
        "{xml}"
    );
}

#[test]
fn the_task_starts_unelevated_at_normal_priority_without_a_time_limit() {
    // Act
    let xml = definition(USER, &action(r"C:\apps\mosaico.exe", "start"));

    // Assert
    assert!(xml.contains("<RunLevel>LeastPrivilege</RunLevel>"), "{xml}");
    assert!(xml.contains("<Priority>4</Priority>"), "{xml}");
    assert!(
        xml.contains("<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>"),
        "{xml}"
    );
    assert!(
        xml.contains("<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>"),
        "{xml}"
    );
    assert!(
        xml.contains("<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>"),
        "{xml}"
    );
    assert!(xml.contains("<Settings>"), "{xml}");
    assert!(!xml.contains("<Delay>"), "starts at logon, without a delay");
}

#[test]
fn characters_with_a_meaning_in_xml_are_escaped() {
    // Arrange
    let action = action(r"C:\R&D <tools>\mosaico.exe", r#""it's" a>b"#);

    // Act
    let xml = definition(USER, &action);

    // Assert
    assert!(
        xml.contains(r"<Command>C:\R&amp;D &lt;tools&gt;\mosaico.exe</Command>"),
        "{xml}"
    );
    assert!(
        xml.contains("<Arguments>&quot;it&apos;s&quot; a&gt;b</Arguments>"),
        "{xml}"
    );
}
