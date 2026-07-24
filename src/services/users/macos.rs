//! macOS: reads the `utmpx` user-accounting database via `getutxent(3)`
//!
//! `libc` already exposes the needed bindings (`getutxent`, `setutxent`, `endutxent`,
//! `struct utmpx`, `USER_PROCESS`) for this target, so this needs no new dependency
//!

use super::UsersInfo;
use crate::core::error::AppError;
use std::ffi::CStr;

/// `collect_users()` reads the list of logged-in users from the `utmpx` database
///
#[allow(clippy::unnecessary_wraps)]
pub(super) fn collect_users() -> Result<UsersInfo, AppError> {
    let mut users = Vec::new();

    // SAFETY: `setutxent()` resets the process-global `utmpx` read cursor. It is paired with
    // the `endutxent()` call in `EndUtxentGuard::drop`, which runs on every exit path below,
    // including early returns and panics.
    unsafe {
        libc::setutxent();
    }
    let _guard = EndUtxentGuard;

    loop {
        // SAFETY: `getutxent()` returns either a null pointer (end of database) or a pointer
        // to a libc-owned static record that remains valid only until the next
        // `getutxent()`/`setutxent()`/`endutxent()` call. We read the fields we need
        // immediately, within this iteration, before looping back and calling it again.
        let entry = unsafe { libc::getutxent() };
        let Some(record) = (unsafe { entry.as_ref() }) else {
            break;
        };

        if record.ut_type != libc::USER_PROCESS {
            continue;
        }

        // SAFETY: `ut_user` is a fixed 32-byte, NUL-padded `char` array per `<utmpx.h>`, so
        // it always contains a valid, in-bounds NUL terminator.
        let user = unsafe { CStr::from_ptr(record.ut_user.as_ptr()) }
            .to_string_lossy()
            .into_owned();

        if !user.is_empty() {
            users.push(user);
        }
    }

    users.sort_unstable();
    users.dedup();
    Ok(UsersInfo { users })
}

/// `EndUtxentGuard` ensures `endutxent()` runs on every return path out of `collect_users()`,
/// closing the database cursor opened by `setutxent()`
///
struct EndUtxentGuard;

impl Drop for EndUtxentGuard {
    /// `drop()` closes the `utmpx` database cursor
    ///
    fn drop(&mut self) {
        // SAFETY: closes a cursor opened by `setutxent()` above; well-defined even if zero
        // entries were read in between.
        unsafe {
            libc::endutxent();
        }
    }
}
