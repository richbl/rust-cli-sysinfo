use std::process;

use super::prelude::*;

/// `UsersInfo` contains the list of logged-in user list collected from `who`
#[derive(Default)]
pub struct UsersInfo {
    pub users: Vec<String>,
}

/// `UsersService` is a struct for collecting and rendering logged-in users
pub struct UsersService;

/// `UsersService` implements the `Service` trait
impl Service for UsersService {
    type Data = UsersInfo;

    /// `collect()` runs `who` and returns a sorted, deduplicated list of usernames
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let output = process::Command::new("who")
            .output()
            .map_err(AppError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::DataUnavailable(format!(
                "who command failed: {}",
                stderr.trim()
            )));
        }

        let stdout = std::str::from_utf8(&output.stdout)
            .map_err(|e| AppError::Parse(format!("Invalid UTF-8 in who output: {e}")))?;

        let mut users: Vec<String> = stdout
            .lines()
            .filter_map(|l| l.split_whitespace().next().map(String::from))
            .collect();

        users.sort_unstable();
        users.dedup();

        Ok(UsersInfo { users })
    }

    /// `render()` renders the list of logged-in users as a comma-separated row
    ///
    fn render(&self, users: &Self::Data, c: &Colors) {
        let user_str = if users.users.is_empty() {
            "Nobody".to_string()
        } else {
            users.users.join(", ")
        };

        print_row("  User(s):", &user_str, &Threshold::None, c);
    }
}
