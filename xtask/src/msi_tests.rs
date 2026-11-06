use super::*;

#[test]
fn a_release_version_is_the_package_version() {
    // Act / Assert
    assert_eq!(msi_version("0.12.2").unwrap(), "0.12.2");
}

#[test]
fn a_prerelease_is_packaged_as_its_release() {
    // Act / Assert
    assert_eq!(msi_version("1.2.3-rc.1").unwrap(), "1.2.3");
    assert_eq!(msi_version("1.2.3+build.5").unwrap(), "1.2.3");
}

#[test]
fn versions_windows_installer_cannot_hold_are_refused() {
    // Act / Assert
    assert!(msi_version("256.0.0").is_err());
    assert!(msi_version("1.2.65536").is_err());
    assert!(msi_version("1.2").is_err());
    assert!(msi_version("1.2.3.4").is_err());
    assert!(msi_version("v1.2.3").is_err());
}

#[test]
fn options_read_a_version_and_skipping_the_build() {
    // Act
    let options = Options::parse(&["--no-build", "--version", "9.9.9"]).unwrap();

    // Assert
    assert_eq!(
        options,
        Options {
            version: Some("9.9.9".into()),
            no_build: true,
        }
    );
}

#[test]
fn a_version_flag_without_a_version_is_refused() {
    // Act / Assert
    assert!(Options::parse(&["--version"]).is_err());
    assert!(Options::parse(&["--sign"]).is_err());
}
