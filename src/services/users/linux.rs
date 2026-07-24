//! Linux: reads `/var/run/utmp` via the `utwt` crate
//!

use super::UsersInfo;
use crate::core::error::AppError;

/// `collect_users()` reads the list of logged-in users from `utmp`
///
pub(super) fn collect_users() -> Result<UsersInfo, AppError> {
    let entries = match utwt::parse_utmp() {
        Ok(e) => e,
        Err(utwt::ParseError::Io(io_err))
            if matches!(
                io_err.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
            ) =>
        {
            // Oh no! `utmp` absent or permission denied? --> degrade gracefully
            return Ok(UsersInfo::default());
        }
        Err(e) => return Err(AppError::from(e)),
    };

    let mut users: Vec<String> = entries
        .into_iter()
        .filter_map(|entry| match entry {
            // user: String is a named field inside UserProcess — no method call needed
            utwt::UtmpEntry::UserProcess { user, .. } if !user.is_empty() => Some(user),
            _ => None,
        })
        .collect();

    users.sort_unstable();
    users.dedup();
    Ok(UsersInfo { users })
}
