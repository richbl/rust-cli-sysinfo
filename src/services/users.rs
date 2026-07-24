use super::prelude::*;

/// `UsersInfo` contains the list of currently logged-in users
///
/// A "logged-in user" means an account with an **active, currently-connected interactive
/// login session** on this machine right now — the same standard the `who(1)` command applies
/// on Linux via `utmp`: someone sitting at the console, or connected in over SSH/RDP, at this
/// moment. It excludes:
///   - system/service accounts that own background daemons but never logged in interactively
///   - accounts that logged in previously but are no longer connected (e.g. a disconnected,
///     backgrounded Remote Desktop session on Windows)
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

    /// `collect()` delegates to the platform-specific implementation selected below
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        platform::collect_users()
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

// Platform-specific collection lives under `src/services/users/`: one file per OS/platform
//
#[cfg(target_os = "linux")]
#[path = "users/linux.rs"]
mod platform;

#[cfg(target_os = "windows")]
#[path = "users/windows.rs"]
mod platform;

#[cfg(target_os = "macos")]
#[path = "users/macos.rs"]
mod platform;

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
compile_error!(
    "the `users` service has no implementation for this target; add src/services/users/<platform>.rs and wire it in via #[cfg] in src/services/users.rs"
);

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
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
mod tests {
    use super::*;

    /// `collect_returns_ok()` asserts that collecting logged-in users returns `Ok` on every
    /// implemented platform (Linux via `/var/run/utmp`, Windows via WTS, macOS via `utmpx`)
    ///
    #[test]
    fn collect_returns_ok() {
        assert!(UsersService.collect().is_ok());
    }

    /// `render_does_not_panic()` asserts that rendering logged-in users does not panic... Phew!
    ///
    #[test]
    fn render_does_not_panic() {
        let data = UsersService.collect().unwrap();
        let _ = UsersService.render(&data);
    }
}
