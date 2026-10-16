use super::*;

#[test]
fn explicit_instances_validate_their_name() {
    // Act / Assert
    assert!(Instance::new(Some("ok_name-2"), "c", "d").is_ok());
    assert!(Instance::new(Some(""), "c", "d").is_err());
    assert!(Instance::new(None, "c", "d").unwrap().is_default());
}

#[test]
fn an_instance_keeps_its_logs_and_pid_file_in_its_own_directories() {
    // Act
    let instance = Instance::new(Some("ci"), "cfg", "data").unwrap();

    // Assert
    assert_eq!(instance.endpoint_name(), "mosaico-ci");
    assert_eq!(instance.log_dir(), Path::new("cfg").join("logs"));
    assert_eq!(instance.pid_path(), Path::new("data").join("mosaico.pid"));
}
