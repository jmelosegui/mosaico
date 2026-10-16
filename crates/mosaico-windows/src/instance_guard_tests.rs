use windows::Win32::Security::{
    ACL, ACL_REVISION, InitializeAcl, InitializeSecurityDescriptor, PSECURITY_DESCRIPTOR,
    SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR, SetSecurityDescriptorDacl,
};

use super::*;

/// The revision `InitializeSecurityDescriptor` takes, which the Windows
/// headers call `SECURITY_DESCRIPTOR_REVISION`.
const SECURITY_DESCRIPTOR_REVISION: u32 = 1;

/// A mutex name of this test process, so test runs in two worktrees
/// at the same time do not take each other's mutex.
fn test_mutex(name: &str) -> String {
    format!("Local\\MosaicoTest_{name}-{}", std::process::id())
}

/// A named instance of this test process, so its lock is the test's own
/// and never the user's daemon's.
fn test_instance(test: &str) -> Instance {
    let name = format!("test-{test}-{}", std::process::id());
    Instance::new(Some(&name), "cfg", "data").unwrap()
}

/// Creates the mutex `name` with an empty access list, so no process,
/// this one included, may open it again, as with the lock of another
/// user's daemon. The mutex lasts as long as the returned handle.
fn hold_unopenable(name: &str) -> HANDLE {
    let mut acl = ACL::default();
    let mut descriptor = SECURITY_DESCRIPTOR::default();
    let descriptor_ptr = PSECURITY_DESCRIPTOR(std::ptr::from_mut(&mut descriptor).cast());

    // SAFETY: the access list and the descriptor are locals that outlive
    // the CreateMutexW call below, which copies them, and the sizes and
    // revisions are the ones these functions expect for an empty list.
    unsafe {
        InitializeAcl(&mut acl, std::mem::size_of::<ACL>() as u32, ACL_REVISION)
            .expect("failed to set up the access list");
        InitializeSecurityDescriptor(descriptor_ptr, SECURITY_DESCRIPTOR_REVISION)
            .expect("failed to set up the security descriptor");
        SetSecurityDescriptorDacl(descriptor_ptr, true, Some(std::ptr::from_ref(&acl)), false)
            .expect("failed to put the access list in the descriptor");
    }
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor_ptr.0,
        bInheritHandle: false.into(),
    };

    // SAFETY: the attributes point at the descriptor above, which is
    // still alive, and the name is a valid wide string.
    unsafe { CreateMutexW(Some(&attributes), false, &HSTRING::from(name)) }
        .expect("failed to create the mutex")
}

#[test]
fn second_acquire_is_rejected() {
    // Arrange
    let name = test_mutex("SecondAcquire");
    let _first = InstanceGuard::acquire_named(&name).expect("first acquire should succeed");

    // Act
    let second = InstanceGuard::acquire_named(&name);

    // Assert
    assert!(second.is_err());
}

#[test]
fn a_second_daemon_is_told_that_one_is_already_running() {
    // Arrange
    let name = test_mutex("AlreadyRunning");
    let _first = InstanceGuard::acquire_named(&name).expect("first acquire should succeed");

    // Act
    let second = InstanceGuard::acquire_named(&name);

    // Assert -- not mistaken for a failed OS call.
    assert!(
        matches!(second, Err(Error::AlreadyRunning)),
        "{:?}",
        second.err()
    );
}

#[test]
fn the_default_instance_keeps_the_usual_mutex() {
    // Arrange
    let default = Instance::new(None, "cfg", "data").unwrap();
    let named = Instance::new(Some("ci"), "cfg", "data").unwrap();

    // Act / Assert
    assert_eq!(mutex_name(&default), "Global\\MosaicoDaemon");
    assert_eq!(mutex_name(&named), "Global\\MosaicoDaemon-ci");
}

#[test]
fn acquire_succeeds_after_release() {
    // Arrange
    let name = test_mutex("AfterRelease");
    let first = InstanceGuard::acquire_named(&name).expect("first acquire should succeed");

    // Act
    drop(first);
    let second = InstanceGuard::acquire_named(&name);

    // Assert
    assert!(second.is_ok());
}

#[test]
fn an_instance_stays_locked_until_its_lock_is_dropped() {
    // Arrange
    let instance = test_instance("locked-until-dropped");
    let lock = NamedMutex
        .acquire(&instance)
        .expect("the first lock should be taken");

    // Act
    let while_held = NamedMutex.acquire(&instance);
    drop(lock);
    let after = NamedMutex.acquire(&instance);

    // Assert
    assert!(
        matches!(while_held, Err(Error::AlreadyRunning)),
        "{while_held:?}"
    );
    assert!(after.is_ok(), "{after:?}");
}

#[test]
fn instances_lock_independently() {
    // Arrange
    let one = test_instance("independent-one");
    let other = test_instance("independent-other");
    let _held = NamedMutex
        .acquire(&one)
        .expect("the first lock should be taken");

    // Act
    let result = NamedMutex.acquire(&other);

    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn instances_with_the_same_name_share_one_lock() {
    // Arrange -- the same name with other directories, as when only
    // MOSAICO_DATA_DIR differs. They would share one IPC endpoint too.
    let name = format!("test-same-name-{}", std::process::id());
    let one = Instance::new(Some(&name), "cfg-one", "data-one").unwrap();
    let other = Instance::new(Some(&name), "cfg-other", "data-other").unwrap();
    let _held = NamedMutex
        .acquire(&one)
        .expect("the first lock should be taken");

    // Act
    let result = NamedMutex.acquire(&other);

    // Assert
    assert!(matches!(result, Err(Error::AlreadyRunning)), "{result:?}");
}

#[test]
fn a_lock_this_process_may_not_open_is_a_platform_error() {
    // Arrange -- held under an access list that lets nobody open it, as
    // the lock of another user's daemon is.
    let name = test_mutex("NotOpenable");
    let held = hold_unopenable(&name);

    // Act
    let result = InstanceGuard::acquire_named(&name);
    // SAFETY: closes the handle created above, which nothing else uses.
    unsafe {
        let _ = CloseHandle(held);
    }

    // Assert -- refused, but not taken for a daemon this process can see,
    // and with the message the daemon has always given for it.
    let error = result.err().expect("the lock should be refused");
    assert!(
        matches!(
            error,
            Error::Platform {
                operation: "create the single instance lock",
                ..
            }
        ),
        "{error:?}"
    );
}
