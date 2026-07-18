use super::prelude::*;

#[cfg(not(target_os = "linux"))]
use crate::constants::NOT_YET_IMPLEMENTED;

/// `UsersInfo` contains the list of logged-in users (Linux only, via `/var/run/utmp`)
/// Currently unclear how to implement this on Windows or macOS, as the concept is not
/// cross-platform
///
#[derive(Default)]
pub struct UsersInfo {
    pub users: Vec<String>,
}

/// `UsersService` is a struct for collecting and rendering logged-in users
pub struct UsersService;

/// `UsersService` implements the `Service` trait
impl Service for UsersService {
    type Data = UsersInfo;

    /// `collect()` delegates to the platform-specific implementation below
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        collect_users()
    }

    /// `render()` renders the list of logged-in users as a comma-separated row
    ///
    fn render(&self, users: &Self::Data) -> Result<RenderedRow, AppError> {
        let user_str = if users.users.is_empty() {
            "Nobody".to_string()
        } else {
            users.users.join(", ")
        };
        Ok(RenderedRow {
            value: user_str,
            threshold: Threshold::None,
        })
    }
}

/// `collect_users()` reads the list of logged-in users
///
#[cfg(target_os = "linux")]
fn collect_users() -> Result<UsersInfo, AppError> {
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

/// `collect_users()` — the non-Linux fallback message...
///
#[cfg(not(target_os = "linux"))]
fn collect_users() -> Result<UsersInfo, AppError> {
    Err(AppError::DataUnavailable(NOT_YET_IMPLEMENTED.into()))
}

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(_ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "USR",
            label: "User(s)",
            description: "Current users",
            sort_order: 70,
        },
        Box::new(UsersService),
    )
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;

    /// `collect_returns_ok()` asserts that collecting logged-in users from `/var/run/utmp`
    /// returns `Ok`
    ///
    #[test]
    fn collect_returns_ok() {
        assert!(UsersService.collect().is_ok());
    }

    /// `render_does_not_panic()` asserts that rendering logged-in users from `/var/run/utmp`
    /// does not panic... Phew!
    ///
    #[test]
    fn render_does_not_panic() {
        let data = UsersService.collect().unwrap();
        let _ = UsersService.render(&data);
    }
}
