use std::path::{Path, PathBuf};

use super::prelude::*;
use crate::constants::{DISK_CRIT_PCT, DISK_WARN_PCT};
use crate::presentation::format::format_size;

/// `DiskInfo` contains disk usage metrics for a single mount point
#[derive(Default, Debug)]
pub struct DiskInfo {
    pub total_kb: u64,
    pub used_kb: u64,
    pub pct: f64,
}

/// `DiskService` collects and renders disk usage for a given mount path
pub struct DiskService {
    pub mount: PathBuf,
}

/// `DiskService` is a struct for collecting and rendering disk usage
impl DiskService {
    /// `new()` creates a new `DiskService`
    ///
    pub fn new(ctx: &ServiceContext) -> Self {
        Self {
            mount: ctx.disk_mount.clone(),
        }
    }
}

/// `is_prefix_of()` returns true if `candidate` is a path-component-wise prefix of `target`
///
fn is_prefix_of(candidate: &Path, target: &Path) -> bool {
    #[cfg(windows)]
    {
        let candidate_lower = candidate.to_string_lossy().to_lowercase();
        let target_lower = target.to_string_lossy().to_lowercase();
        Path::new(&target_lower).starts_with(Path::new(&candidate_lower))
    }
    #[cfg(not(windows))]
    {
        target.starts_with(candidate)
    }
}

/// `best_matching_mount()` finds the mount point among `candidates`
///
fn best_matching_mount<'a>(
    target: &Path,
    candidates: impl Iterator<Item = &'a Path>,
) -> Option<&'a Path> {
    candidates
        .filter(|candidate| is_prefix_of(candidate, target))
        .max_by_key(|candidate| candidate.as_os_str().len())
}

/// `DiskService` implements the `Service` trait
impl Service for DiskService {
    type Data = DiskInfo;

    /// `collect()` enumerates disks and resolves the configured path to its enclosing filesystem
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        if !self.mount.try_exists().unwrap_or(false) {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such file or directory: {}", self.mount.display()),
            )));
        }

        let disks = sysinfo::Disks::new_with_refreshed_list();
        let mount_points: Vec<&Path> = disks
            .list()
            .iter()
            .map(sysinfo::Disk::mount_point)
            .collect();

        let Some(best_mount) = best_matching_mount(&self.mount, mount_points.into_iter()) else {
            return Err(AppError::DataUnavailable(format!(
                "could not resolve a filesystem for {}",
                self.mount.display()
            )));
        };

        let disk = disks
            .list()
            .iter()
            .find(|d| d.mount_point() == best_mount)
            .expect("best_matching_mount() must return a mount point taken from `disks`");

        let total_kb = disk.total_space() / 1024;
        let free_kb = disk.available_space() / 1024;

        // `df` calculates used as: total - free
        let used_kb = total_kb.saturating_sub(free_kb);

        #[allow(clippy::cast_precision_loss)]
        let pct = if total_kb > 0 {
            (used_kb as f64 / total_kb as f64) * 100.0
        } else {
            0.0
        };

        Ok(DiskInfo {
            total_kb,
            used_kb,
            pct,
        })
    }

    /// `render()` renders disk usage as a percentage with used/total sizes
    ///
    fn render(&self, disk: &Self::Data) -> Result<RenderedRow, AppError> {
        let (value, threshold) = if disk.total_kb == 0 {
            // Oh wow! A real disk was resolved, but reporting zero total space which could
            // conceivably occur
            ("n/a".to_string(), Threshold::None)
        } else {
            // display() handles formatting potential non-UTF-8 characters
            let text = format!(
                "{:.1}% ({}/{}) of {}",
                disk.pct,
                format_size(disk.used_kb),
                format_size(disk.total_kb),
                self.mount.display()
            );
            (
                text,
                Threshold::Check {
                    value: disk.pct,
                    warn: DISK_WARN_PCT,
                    crit: DISK_CRIT_PCT,
                },
            )
        };

        Ok(RenderedRow { value, threshold })
    }
}

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "DSKU",
            label: "Disk usage",
            description: "Disk usage % (Used/Total)",
            sort_order: 120,
        },
        Box::new(DiskService::new(ctx)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `definitely_missing_path()` returns an OS-appropriate absolute path
    ///
    fn definitely_missing_path() -> PathBuf {
        let dir = tempfile::tempdir().expect("failed to create secure temp dir");
        dir.path().to_path_buf()
    }

    /// `collect_missing_path_returns_io_not_found_error()` is the specific regression this test
    /// module exists for: a nonexistent path (bad syntax, wrong drive letter, typo'd mount)
    ///
    #[test]
    fn collect_missing_path_returns_io_not_found_error() {
        let svc = DiskService {
            mount: definitely_missing_path(),
        };

        let result = svc.collect();
        assert!(
            matches!(&result, Err(AppError::Io(_))),
            "expected Err(AppError::Io(NotFound)), got {result:?}"
        );

        if let Err(AppError::Io(e)) = result {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        }
    }

    /// `collect_existing_path_returns_ok_with_nonzero_total()` asserts that a path that definitely
    /// exists
    ///
    #[test]
    fn collect_existing_path_returns_ok_with_nonzero_total() {
        let tmp = tempfile::tempdir().expect("failed to create secure temp dir");
        let svc = DiskService {
            mount: tmp.path().to_path_buf(),
        };

        let result = svc.collect();
        assert!(
            result.is_ok(),
            "collect() must succeed for a real, existing path"
        );
        assert!(
            result.unwrap().total_kb > 0,
            "a real filesystem must report nonzero total space"
        );
    }

    // --- render() : pure formatting, no real disks needed ---

    /// `render_zero_total_shows_plain_na()` asserts the "resolved a disk, it's just empty" case
    /// renders as an uncolored `n/a`
    ///
    #[test]
    fn render_zero_total_shows_plain_na() {
        let svc = DiskService {
            mount: PathBuf::from("/anywhere"),
        };
        let data = DiskInfo::default();

        let row = svc.render(&data).unwrap();
        assert_eq!(row.value, "n/a");
        assert!(matches!(row.threshold, Threshold::None));
    }

    /// `assert_is_check_threshold_matching()` asserts `threshold` is a `Threshold::Check` whose
    /// fields match the given expected values
    ///
    fn assert_is_check_threshold_matching(
        threshold: &Threshold,
        expected_value: f64,
        expected_warn: f64,
        expected_crit: f64,
    ) {
        assert!(
            matches!(threshold, Threshold::Check { .. }),
            "expected Threshold::Check, got a different variant"
        );

        if let Threshold::Check { value, warn, crit } = threshold {
            assert!((value - expected_value).abs() < f64::EPSILON);
            assert!((warn - expected_warn).abs() < f64::EPSILON);
            assert!((crit - expected_crit).abs() < f64::EPSILON);
        }
    }

    /// `render_nonzero_total_uses_check_threshold_with_disk_constants()` asserts a real result
    /// renders with the disk-specific warn/crit thresholds
    ///
    #[test]
    fn render_nonzero_total_uses_check_threshold_with_disk_constants() {
        let svc = DiskService {
            mount: PathBuf::from("/anywhere"),
        };
        let data = DiskInfo {
            total_kb: 1_000_000,
            used_kb: 850_000,
            pct: 85.0,
        };

        let row = svc.render(&data).unwrap();
        assert!(row.value.contains("85.0%"));
        assert!(row.value.contains("/anywhere"));
        assert_is_check_threshold_matching(&row.threshold, 85.0, DISK_WARN_PCT, DISK_CRIT_PCT);
    }

    /// `picks_longest_enclosing_mount()` asserts the closest (longest-prefix) mount wins over a
    /// shallower one that also matches
    ///
    #[test]
    #[cfg(not(windows))]
    fn picks_longest_enclosing_mount() {
        let root = Path::new("/");
        let home = Path::new("/home");
        let target = Path::new("/home/user/docs");

        let candidates = [root, home];
        let best = best_matching_mount(target, candidates.into_iter());
        assert_eq!(best, Some(home));
    }

    /// `falls_back_to_root_when_no_deeper_mount_matches()` asserts root still matches when it's
    /// the only enclosing candidate
    ///
    #[test]
    #[cfg(not(windows))]
    fn falls_back_to_root_when_no_deeper_mount_matches() {
        let root = Path::new("/");
        let boot = Path::new("/boot");
        let target = Path::new("/var/log");

        let candidates = [root, boot];
        let best = best_matching_mount(target, candidates.into_iter());
        assert_eq!(best, Some(root));
    }

    /// `sibling_path_with_shared_string_prefix_does_not_false_match()` checks that
    /// `/homework` must NOT match candidate `/home`
    ///
    #[test]
    #[cfg(not(windows))]
    fn sibling_path_with_shared_string_prefix_does_not_false_match() {
        let home = Path::new("/home");
        let target = Path::new("/homework");

        let candidates = [home];
        let best = best_matching_mount(target, candidates.into_iter());
        assert_eq!(best, None);
    }

    /// `no_candidate_matches_returns_none()` asserts an unrelated set of mounts correctly
    /// resolves to no match rather than a wrong guess
    ///
    #[test]
    #[cfg(not(windows))]
    fn no_candidate_matches_returns_none() {
        let home = Path::new("/home");
        let boot = Path::new("/boot");
        let target = Path::new("/mnt/external");

        let candidates = [home, boot];
        let best = best_matching_mount(target, candidates.into_iter());
        assert_eq!(best, None);
    }

    /// `windows_drive_letter_matches_subdirectory()` asserts a plain drive-letter mount matches
    /// a subdirectory beneath it
    ///
    #[test]
    #[cfg(windows)]
    fn windows_drive_letter_matches_subdirectory() {
        let c_drive = Path::new("C:\\");
        let target = Path::new("C:\\Users\\foo");

        let candidates = [c_drive];
        let best = best_matching_mount(target, candidates.into_iter());
        assert_eq!(best, Some(c_drive));
    }

    /// `windows_drive_letter_case_insensitive_match()` asserts that a lowercase-typed drive letter
    /// must still match an uppercase reported mount point
    ///
    #[test]
    #[cfg(windows)]
    fn windows_drive_letter_case_insensitive_match() {
        let reported_mount = Path::new("E:\\");
        let user_typed_target = Path::new("e:\\some-folder");

        let candidates = [reported_mount];
        let best = best_matching_mount(user_typed_target, candidates.into_iter());
        assert_eq!(best, Some(reported_mount));
    }

    /// `windows_wrong_drive_letter_does_not_match_a_different_drive()` guards against the
    /// case-insensitive comparison matching an unrelated drive
    ///
    #[test]
    #[cfg(windows)]
    fn windows_wrong_drive_letter_does_not_match_a_different_drive() {
        let c_drive = Path::new("C:\\");
        let target = Path::new("D:\\data");

        let candidates = [c_drive];
        let best = best_matching_mount(target, candidates.into_iter());
        assert_eq!(best, None);
    }
}
