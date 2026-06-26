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

    fn collect(&self) -> Result<Self::Data, AppError> {
        let mut file = File::open("/var/run/utmp").map_err(AppError::Io)?;
        let mut users = Vec::new();

        while let Some(entry) = read_utmpx_entry(&mut file).map_err(AppError::Io)? {
            if entry.ut_type == libc::USER_PROCESS {
                let user = c_char_array_to_string(&entry.ut_user);
                if !user.is_empty() {
                    users.push(user);
                }
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

/// `read_utmpx_entry()` reads the next `utmpx` record from `file`
///
fn read_utmpx_entry(file: &mut File) -> Result<Option<libc::utmpx>, io::Error> {
    // SAFETY: `libc::utmpx` is a C POD struct (all fields are integers or fixed `c_char`arrays)
    // Every possible bit pattern is a valid, initialized value, so byte-filling the struct via
    // `read_exact` and immediately calling `assume_init` is sound
    unsafe {
        let mut entry = MaybeUninit::<libc::utmpx>::uninit();
        let buf = std::slice::from_raw_parts_mut(
            entry.as_mut_ptr().cast::<u8>(),
            std::mem::size_of::<libc::utmpx>(),
        );
        match file.read_exact(buf) {
            Ok(()) => Ok(Some(entry.assume_init())),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e),
        }
    }
}
