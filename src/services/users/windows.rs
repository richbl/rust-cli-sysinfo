//! Windows: enumerates Remote Desktop Services (WTS) sessions
//!
//! # Why WTS
//!
//! Filtering to `WTSActive` naturally excludes service accounts, since Windows services
//! don't run in an interactive WTS session.

use super::UsersInfo;
use crate::core::error::AppError;
use std::ffi::c_void;
use windows::Win32::System::RemoteDesktop::{
    WTS_CURRENT_SERVER_HANDLE, WTS_SESSION_INFOW, WTSActive, WTSEnumerateSessionsW, WTSFreeMemory,
    WTSQuerySessionInformationW, WTSUserName,
};
use windows::core::PWSTR;

/// `collect_users()` enumerates active WTS sessions and returns their logged-on user names
///
pub(super) fn collect_users() -> Result<UsersInfo, AppError> {
    let mut users: Vec<String> = active_session_ids()?
        .into_iter()
        .filter_map(session_user_name)
        .filter(|user| !user.is_empty())
        .collect();

    users.sort_unstable();
    users.dedup();
    Ok(UsersInfo { users })
}

/// `active_session_ids()` returns the session IDs currently in the `WTSActive` connection
/// state — i.e., sessions with a live, currently-connected interactive logon
///
fn active_session_ids() -> Result<Vec<u32>, AppError> {
    let mut session_info_ptr: *mut WTS_SESSION_INFOW = std::ptr::null_mut();
    let mut count: u32 = 0;

    // SAFETY: `WTSEnumerateSessionsW` allocates `count` contiguous `WTS_SESSION_INFOW` records
    // on success and hands ownership of that buffer to us; `_guard` (below) releases it on
    // every subsequent exit path, including panics.
    unsafe {
        WTSEnumerateSessionsW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            0,
            1,
            &raw mut session_info_ptr,
            &raw mut count,
        )?;
    }

    // Guarantees WTSFreeMemory(session_info_ptr) runs when this function returns, however it
    // returns — see `WtsMemory` below.
    let _guard = WtsMemory(session_info_ptr.cast());

    // SAFETY: `session_info_ptr` is non-null and valid for `count` reads of `WTS_SESSION_INFOW`
    // immediately after the successful call above, and remains so for `_guard`'s lifetime.
    let session_ids: Vec<u32> =
        unsafe { std::slice::from_raw_parts(session_info_ptr, count as usize) }
            .iter()
            .filter(|session| session.State == WTSActive)
            .map(|session| session.SessionId)
            .collect();

    Ok(session_ids)
}

/// `session_user_name()` resolves the account name logged into `session_id`, or `None` if the
/// session has no associated user (e.g., it disconnected in the moment between calls)
///
fn session_user_name(session_id: u32) -> Option<String> {
    let mut buffer_ptr = PWSTR::null();
    let mut bytes_returned: u32 = 0;

    // SAFETY: `buffer_ptr`/`bytes_returned` are out-parameters populated by the call; on
    // success `buffer_ptr` points to a buffer we own, released below by `_guard`.
    let query_result = unsafe {
        WTSQuerySessionInformationW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            session_id,
            WTSUserName,
            &raw mut buffer_ptr,
            &raw mut bytes_returned,
        )
    };

    if query_result.is_err() {
        return None;
    }

    // Guarantees WTSFreeMemory(buffer_ptr) runs when this function returns, however it
    // returns — see `WtsMemory` below.
    let _guard = WtsMemory(buffer_ptr.0.cast());

    // SAFETY: on success, `buffer_ptr` points to a valid, null-terminated UTF-16 string per the
    // documented `WTSUserName` info-class contract, and remains so for `_guard`'s lifetime.
    unsafe { buffer_ptr.to_string() }.ok()
}

/// `WtsMemory` releases a buffer allocated by a `WTSEnumerateSessionsW`/
/// `WTSQuerySessionInformationW` call via `WTSFreeMemory` when dropped — guaranteeing cleanup
/// on every exit path out of the function that created it, including early returns and panics,
/// rather than relying on a manually-placed free call right before each `return`. Shared by
/// both call sites above since both ultimately free through the same Win32 API.
struct WtsMemory(*mut c_void);

impl Drop for WtsMemory {
    /// `drop()` releases the wrapped WTS-allocated buffer
    ///
    fn drop(&mut self) {
        // SAFETY: `self.0` was allocated by a prior WTS Enumerate/Query call and is only ever
        // constructed here after such a call succeeds (see both call sites above), so it has
        // not yet been freed; `WTSFreeMemory` is documented as safe to call exactly once per
        // allocation.
        unsafe {
            WTSFreeMemory(self.0);
        }
    }
}
