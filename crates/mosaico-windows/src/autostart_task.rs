//! The logon task that starts mosaico, through the Task Scheduler's COM
//! API (`ITaskService`).
//!
//! The task lives in the root folder of the Task Scheduler Library: a
//! user who is not an administrator may add tasks there, and a folder
//! one user creates may refuse another. Task names are shared by every
//! user of the machine, so the name holds the user's security
//! identifier.

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, VARIANT_FALSE};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize,
};
use windows::Win32::System::TaskScheduler::{
    IExecAction, IRegisteredTask, ITaskFolder, ITaskService, TASK_CREATE_OR_UPDATE,
    TASK_LOGON_INTERACTIVE_TOKEN, TaskScheduler,
};
use windows::Win32::System::Variant::VARIANT;
use windows::core::{BSTR, HRESULT, Interface};

use mosaico_core::Result;

use crate::error::OrPlatform;

/// What a registered logon task runs, and whether it is on.
pub struct Registered {
    /// The program of its first action, empty when that runs no program.
    pub program: String,
    /// The arguments of that program, as one line.
    pub arguments: String,
    /// Whether the task is enabled. The user can disable it in the Task
    /// Scheduler, and then it does not start.
    pub enabled: bool,
}

/// The logon task of one user.
pub struct LogonTask {
    /// The name of the task in the root folder.
    name: BSTR,
}

impl LogonTask {
    /// The logon task of the user with the security identifier `user`.
    pub fn of(user: &str) -> Self {
        Self {
            name: BSTR::from(format!("Mosaico autostart for {user}")),
        }
    }

    /// Reads the task, or `None` when it is not registered.
    pub fn read(&self) -> Result<Option<Registered>> {
        const OPERATION: &str = "read the autostart task";
        let library = Library::open(OPERATION)?;
        // SAFETY: GetTask on a connected root folder, with a BSTR name.
        let task = match unsafe { library.root.GetTask(&self.name) } {
            Ok(task) => task,
            Err(e) if is_missing(e.code()) => return Ok(None),
            Err(e) => return Err(e).or_platform(OPERATION),
        };
        // SAFETY: Enabled on a task GetTask returned.
        let enabled = unsafe { task.Enabled() }.or_platform(OPERATION)?;
        let (program, arguments) = first_program(&task).or_platform(OPERATION)?;
        Ok(Some(Registered {
            program,
            arguments,
            enabled: enabled != VARIANT_FALSE,
        }))
    }

    /// Registers the task from its XML `definition`, replacing the task
    /// of that name if there is one.
    pub fn register(&self, definition: &str) -> Result<()> {
        const OPERATION: &str = "register the autostart task";
        let library = Library::open(OPERATION)?;
        let none = VARIANT::default();
        // SAFETY: RegisterTask on a connected root folder. The name and
        // definition are BSTRs, and the user, password and security
        // descriptor are empty, which registers the task for the user
        // its definition names, with that user's logon token.
        unsafe {
            library.root.RegisterTask(
                &self.name,
                &BSTR::from(definition),
                TASK_CREATE_OR_UPDATE.0,
                &none,
                &none,
                TASK_LOGON_INTERACTIVE_TOKEN,
                &none,
            )
        }
        .map(drop)
        .or_platform(OPERATION)
    }

    /// Deletes the task. Succeeds when it is not registered.
    pub fn delete(&self) -> Result<()> {
        const OPERATION: &str = "remove the autostart task";
        let library = Library::open(OPERATION)?;
        // SAFETY: DeleteTask on a connected root folder, with a BSTR name.
        match unsafe { library.root.DeleteTask(&self.name, 0) } {
            Err(e) if is_missing(e.code()) => Ok(()),
            result => result.or_platform(OPERATION),
        }
    }
}

/// The program and arguments of the first action of `task`, or empty
/// ones when that action runs no program, or the task has no action.
fn first_program(task: &IRegisteredTask) -> windows::core::Result<(String, String)> {
    // SAFETY: Definition and Actions on a task GetTask returned.
    let actions = unsafe { task.Definition()?.Actions()? };
    let mut count = 0;
    // SAFETY: Count writes the number of actions to `count`.
    unsafe { actions.Count(&mut count)? };
    if count < 1 {
        return Ok(Default::default());
    }
    // SAFETY: Actions are numbered from 1, and there is at least one.
    let Ok(exec) = unsafe { actions.get_Item(1)? }.cast::<IExecAction>() else {
        return Ok(Default::default());
    };
    let (mut program, mut arguments) = (BSTR::new(), BSTR::new());
    // SAFETY: Path and Arguments each write a BSTR the caller owns,
    // which `BSTR` frees when dropped.
    unsafe {
        exec.Path(&mut program)?;
        exec.Arguments(&mut arguments)?;
    }
    Ok((program.to_string(), arguments.to_string()))
}

/// Whether a failure means the task is not there.
fn is_missing(code: HRESULT) -> bool {
    code == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0)
}

/// The root folder of the Task Scheduler Library, connected on this
/// thread.
struct Library {
    /// The root folder. Declared first so it is released before COM is
    /// left.
    root: ITaskFolder,
    /// Whether this thread entered COM to connect, and leaves it after.
    _apartment: Apartment,
}

impl Library {
    /// Connects to the Task Scheduler on this computer as the user who
    /// runs mosaico, and opens its root folder.
    fn open(operation: &'static str) -> Result<Self> {
        let apartment = Apartment::enter();
        let root = root_folder().or_platform(operation)?;
        Ok(Self {
            root,
            _apartment: apartment,
        })
    }
}

/// The root folder of the Task Scheduler Library on this computer, seen
/// by the user who runs mosaico. COM must be entered on this thread.
fn root_folder() -> windows::core::Result<ITaskFolder> {
    let none = VARIANT::default();
    // SAFETY: The caller entered COM on this thread. Connect with empty
    // arguments connects to the local computer as the current user.
    unsafe {
        let service: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
        service.Connect(&none, &none, &none, &none)?;
        service.GetFolder(&BSTR::from("\\"))
    }
}

/// COM, entered on this thread for as long as it lives.
struct Apartment {
    /// Whether this entered COM, and so must leave it. It did not when
    /// the thread is already in another kind of apartment, and COM works
    /// there as it is.
    entered: bool,
}

impl Apartment {
    fn enter() -> Self {
        // SAFETY: CoInitializeEx may be called on any thread. A thread
        // already in COM gets S_FALSE, or RPC_E_CHANGED_MODE for another
        // kind of apartment.
        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        Self {
            entered: result.is_ok(),
        }
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.entered {
            // SAFETY: Paired with the CoInitializeEx in `enter` that
            // succeeded, on the same thread.
            unsafe { CoUninitialize() };
        }
    }
}

#[cfg(test)]
#[path = "autostart_task_tests.rs"]
mod tests;
