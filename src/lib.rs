//! # winerr
//!
//! A tiny crate for retrieving and formatting Windows error messages.
//! 
//! Example usages:
//! 
//! ```rust
//! use winerr::{Error, last_error};
//! 
//! // Get last error and display it
//! let error = last_error();
//! println!("{}", error);
//! 
//! // Get last error and check the code
//! let error = Error::last();
//! println!("{}", error.code());
//! 
//! // Create an error from a code
//! let error = Error::with_code(0);
//! assert_eq!(0, error.code());
//! ```
//! 
#![cfg(windows)]
#![warn(missing_docs)]

use std::fmt::{self, Debug, Display, Formatter};
use std::mem::MaybeUninit;

use winapi::{
    shared::{
        ntdef::NULL,
        winerror::HRESULT_CODE
    },
    um::{
        errhandlingapi::GetLastError,
        winbase::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS},
    },
};

/// A Windows API Error
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Error {
    code: u32,
}
/// Retrieve the last error. Equivilent to windows API call GetLastError().
pub fn last_error() -> Error {
    let code = unsafe { GetLastError() };
    Error::with_code(code)
}
/// Creates an error from the HRESULT value.
pub fn from_hresult(hr: i32) -> Error {
    let code = HRESULT_CODE(hr) as u32;
    Error::with_code(code)
}
impl Error {
    /// Retrieve the last error. Equivilent to windows API call GetLastError().
    pub fn last() -> Self {
        last_error()
    }
    /// Returns the error code
    pub fn code(self) -> u32 {
        self.code
    }
    /// Creates an error with the specified code.
    pub fn with_code(code: u32) -> Self {
        Self { code }
    }
    /// Creates an error from the HRESULT value.
    pub fn from_hresult(hr: i32) -> Self {
        from_hresult(hr)
    }
}

// TODO: fmt with user-provided args

fn fmt_error(code: u32) -> Option<String> {
    const FLAGS: u32 = FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS;
    // Longest error message I can find requires length of 419
    const BUF_SIZE: usize = 420;
    let mut buf = MaybeUninit::<[u16; BUF_SIZE]>::uninit();
    let buf_ptr: *mut u16 = buf.as_mut_ptr().cast();
    unsafe {
        let len = FormatMessageW(
            FLAGS,
            NULL, // source (fmt string)
            code, // msg id
            0,    // lang id
            buf_ptr,
            BUF_SIZE as u32,
            NULL as _, // fmt arguments
        );
        if len == 0 {
            None
        } else {
            let slice = std::slice::from_raw_parts(buf_ptr, len as usize);
            Some(String::from_utf16_lossy(slice))
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(s) = fmt_error(self.code()) {
            write!(f, "{}", s.trim())
        } else {
            // This branch should never happen unless the
            // error code is not a valid Windows message.
            let fmt_err = last_error().code();
            if let Some(s) = fmt_error(fmt_err) {
                write!(
                    f,
                    "Error code {} (could not format due to internal error: {} - {})",
                    self.code(),
                    fmt_err,
                    s.trim()
                )
            } else {
                write!(
                    f,
                    "Error code {} (could not format due to internal error code: {})",
                    self.code(),
                    fmt_err
                )
            }
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> Self {
        Self::from_raw_os_error(e.code() as i32)
    }
}

#[test]
fn test_fmt() {
    let err = Error::with_code(0);
    assert_eq!(format!("{}", err), "The operation completed successfully.");
    let err = Error::with_code(1);
    assert_eq!(format!("{}", err), "Incorrect function.");
    let err = Error::with_code(52);
    assert_eq!(format!("{}", err), "You were not connected because a duplicate name exists on the network. If joining a domain, go to System in Control Panel to change the computer name and try again. If joining a workgroup, choose another workgroup name.");
    let err = Error::with_code(192);
    assert_eq!(format!("{}", err), "The operating system cannot run %1.");
    let err = Error::with_code(560);
    assert_eq!(format!("{}", err), "Indicates that an attempt was made to assign protection to a file system file or directory and one of the SIDs in the security descriptor could not be translated into a GUID that could be stored by the file system.\r\nThis causes the protection attempt to fail, which may cause a file creation attempt to fail.");
    let err = Error::with_code(609);
    assert_eq!(format!("{}", err), "{Invalid DLL Entrypoint}\r\nThe dynamic link library %hs is not written correctly. The stack pointer has been left in an inconsistent state.\r\nThe entrypoint should be declared as WINAPI or STDCALL. Select YES to fail the DLL load. Select NO to continue execution. Selecting NO may cause the application to operate incorrectly.");
    let err = Error::with_code(1290);
    assert_eq!(format!("{}", err), "The service start failed since one or more services in the same process have an incompatible service SID type setting. A service with restricted service SID type can only coexist in the same process with other services with a restricted SID type. If the service SID type for this service was just configured, the hosting process must be restarted in order to start this service.");
    // The longest error message (that I could find)
    let err = Error::with_code(6719);
    assert_eq!(format!("{}", err), "The object specified could not be created or opened, because its associated TransactionManager is not online.  The TransactionManager must be brought fully Online by calling RecoverTransactionManager to recover to the end of its LogFile before objects in its Transaction or ResourceManager namespaces can be opened.  In addition, errors in writing records to its LogFile can cause a TransactionManager to go offline.");
    // A non-existant error code
    let err = Error::with_code(15999);
    assert_eq!(format!("{}", err), "Error code 15999 (could not format due to internal error: 317 - The system cannot find message text for message number 0x%1 in the message file for %2.)");
}
