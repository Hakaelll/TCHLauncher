use std::{io, mem, os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle}, process::{Child, Command}, ptr};
use windows_sys::Win32::System::JobObjects::{AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject, JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE};

pub fn spawn(command: &mut Command) -> io::Result<(Child, OwnedHandle)> {
    // Only this launcher holds the non-inheritable job handle, including in tray mode.
    let raw = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
    if raw.is_null() { return Err(io::Error::last_os_error()); }
    let job = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    if unsafe { SetInformationJobObject(job.as_raw_handle(), JobObjectExtendedLimitInformation, ptr::from_ref(&limits).cast(), mem::size_of_val(&limits) as u32) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut child = command.spawn()?;
    if unsafe { AssignProcessToJobObject(job.as_raw_handle(), child.as_raw_handle()) } == 0 {
        let error = io::Error::last_os_error();
        // Do not publish an uncontrolled server if Windows refuses containment.
        if child.try_wait()?.is_none() { child.kill()?; }
        child.wait()?;
        return Err(error);
    }
    Ok((child, job))
}
