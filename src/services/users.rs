use std::fs::File;
use std::io::{self, Read};
use std::mem::MaybeUninit;

use super::prelude::*;
use crate::core::utils::c_char_array_to_string;

/// `UsersInfo` contains the list of logged-in user list collected from `/var/run/utmp`
#[derive(Default)]
pub struct UsersInfo {
    pub users: Vec<String>,
}

/// `UsersService` is a struct for collecting and rendering logged-in users
pub struct UsersService;

/// `UsersService` implements the `Service` trait
impl Service for UsersService {
    type Data = UsersInfo;

    /// `collect()` reads `/var/run/utmp` natively and returns a sorted, deduplicated list of usernames
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let mut file = File::open("/var/run/utmp").map_err(AppError::Io)?;
        let mut users = Vec::new();

        loop {
            let mut entry = MaybeUninit::<libc::utmpx>::uninit();

            // `libc::utmpx` is a POD struct: we create a byte slice over its memory to read into it
            let buf = unsafe {
                std::slice::from_raw_parts_mut(
                    entry.as_mut_ptr().cast::<u8>(),
                    std::mem::size_of::<libc::utmpx>(),
                )
            };

            match file.read_exact(buf) {
                Ok(()) => {
                    // We just read exactly `size_of::<libc::utmpx>()` bytes into `entry`
                    let entry = unsafe { entry.assume_init() };

                    // Skip entries that do not represent an active user session
                    if entry.ut_type != libc::USER_PROCESS {
                        continue;
                    }

                    let user = c_char_array_to_string(&entry.ut_user);
                    if !user.is_empty() {
                        users.push(user);
                    }
                }
                // Break on EOF (which includes cleanly finishing the file or hitting a partial
                // record at the end)
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(AppError::Io(e)),
            }
        }

        users.sort_unstable();
        users.dedup();

        Ok(UsersInfo { users })
    }

    /// `render()` renders the list of logged-in users as a comma-separated row
    ///
    fn render(&self, label: &str, users: &Self::Data, c: &Colors) {
        let user_str = if users.users.is_empty() {
            "Nobody".to_string()
        } else {
            users.users.join(", ")
        };

        print_row(label, &user_str, &Threshold::None, c);
    }
}
