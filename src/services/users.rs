use std::process;

use super::Service;
use crate::core::error::AppError;
use crate::presentation::colors::{Colors, Threshold};
use crate::presentation::format::print_row;

/// Logged-in user list collected from `who`
pub struct UsersInfo {
    pub users: Vec<String>,
}

/// Service for collecting and rendering currently logged-in users
pub struct UsersService;

/// Collects and renders currently logged-in users
impl Service for UsersService {
    type Data = UsersInfo;

    /// `collect()` runs `who` and returns a sorted, deduplicated list of usernames
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let Ok(output) = process::Command::new("who").output() else {
            return Ok(UsersInfo { users: vec![] });
        };

        let stdout = std::str::from_utf8(&output.stdout).unwrap_or("");
        let mut users: Vec<String> = stdout
            .lines()
            .filter_map(|l| l.split_whitespace().next().map(String::from))
            .collect();

        // Sort and deduplicate the list of users
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
