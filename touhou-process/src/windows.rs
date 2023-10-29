use core::ffi::c_void;
use std::num::NonZeroUsize;
use std::os::windows::io::AsRawHandle;
mod windows {
    pub(crate) use windows::Win32::Foundation::HANDLE;
    pub(crate) use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
    pub(crate) use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION,
        PROCESS_VM_READ, PROCESS_VM_WRITE,
    };
}

use super::ProcessHandle as WrappedHandle;

pub(crate) type Pid = u32;
pub(crate) type ProcessHandle = windows::HANDLE;

pub(crate) fn try_into_process_handle(pid: Pid) -> std::io::Result<ProcessHandle> {
    unsafe {
        windows::OpenProcess(
            windows::PROCESS_CREATE_THREAD
                | windows::PROCESS_QUERY_INFORMATION
                | windows::PROCESS_VM_READ,
            false,
            pid,
        )
        .map_err(From::from)
    }
}

pub(crate) fn pid_from_u32(value: u32) -> Pid {
    value
}

pub(crate) fn pid_to_u32(value: Pid) -> u32 {
    value
}

pub(crate) unsafe fn read_unsafe<T: ?Sized>(
    handle: ProcessHandle,
    addr: NonZeroUsize,
    dest: &mut T,
) -> std::io::Result<()> {
    let sz = std::mem::size_of_val(dest);
    let dest = (dest as *mut T).cast();

    if sz > 0 {
        if windows::ReadProcessMemory(handle, addr.get() as *const c_void, dest, sz, None) == false
        {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

impl WrappedHandle {
    pub fn from_child(child: std::process::Child) -> std::io::Result<Self> {
        Ok(Self(windows::HANDLE(child.as_raw_handle() as isize)))
    }
}
