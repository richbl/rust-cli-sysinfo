//! Linux System Dashboard
//! Reads metrics natively from /proc and Linux system calls
//!
//! Flags: [-d|--disk <path>] [-c|--cpu-sample-rate <ms>] [-n, --no-clear] [-o,--no-color] [-h|--help]

use std::{
    collections::BTreeSet,
    env, fs,
    fs::File,
    io::{self, BufRead},
    io::{BufReader, IsTerminal},
    path::Path,
    process, thread,
    time::Duration,
};

use pci_ids::{Device, FromId, Vendor};

const VERSION: &str = "0.1.0";
const SEP: &str = "─────────────────────────────────────────────────────────";

// Command-line options struct
//
struct Opts {
    clear: bool,
    color: bool,
    disk_mount: String,
    cpu_sample_ms: u64,
}

// Opts::from_args() parses command-line arguments and returns an Opts struct with the appropriate
//
impl Opts {
    fn from_args() -> Self {
        // Defaults
        let mut clear = true;
        let mut color = io::stdout().is_terminal();
        let mut disk_mount = "/home".to_string();
        let mut cpu_sample_ms = 250;

        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-n" | "--no-clear" => clear = false,
                "-o" | "--no-color" => color = false,
                "-d" | "--disk" => {
                    disk_mount = args.next().unwrap_or_else(|| {
                        eprintln!("Error: --disk requires a path argument (e.g., --disk /var)");
                        process::exit(1);
                    });
                }
                "-c" | "--cpu-sample-rate" => {
                    let val = args.next().unwrap_or_else(|| {
                        eprintln!(
                            "Error: --cpu-sample-rate requires a numeric argument (e.g., -c 250)"
                        );
                        process::exit(1);
                    });
                    cpu_sample_ms = val.parse::<u64>().ok().filter(|&ms| ms > 0).unwrap_or_else(|| {
                        eprintln!("Error: --cpu-sample-rate requires a positive integer (e.g., -c 250)");
                        process::exit(1);
                    });
                }
                "-h" | "--help" => {
                    print_usage();
                    process::exit(0);
                }
                other => {
                    eprintln!("\nUnknown option '{other}'");
                    print_usage();
                    process::exit(1);
                }
            }
        }
        Self {
            clear,
            color,
            disk_mount,
            cpu_sample_ms,
        }
    }
}

// Dashboard presentation structs
//
struct Colors {
    red: &'static str,
    green: &'static str,
    yellow: &'static str,
    cyan: &'static str,
    bold: &'static str,
    reset: &'static str,
}

// Colors::new() initializes ANSI color codes if enabled, or empty strings if disabled
//
impl Colors {
    const fn new(enabled: bool) -> Self {
        if enabled {
            Self {
                red: "\x1b[0;31m",
                green: "\x1b[0;32m",
                yellow: "\x1b[0;33m",
                cyan: "\x1b[0;36m",
                bold: "\x1b[1m",
                reset: "\x1b[0m",
            }
        } else {
            Self {
                red: "",
                green: "",
                yellow: "",
                cyan: "",
                bold: "",
                reset: "",
            }
        }
    }
}

// Threshold enum represents the logic for determining color thresholds for metrics

enum Threshold {
    None,
    Check { value: f64, warn: f64, crit: f64 },
}

// Domain models for components of the dashboard
//
struct SystemInfo {
    hostname: String,
    cpu_model: Option<String>,
    gpu_model: Vec<String>,
    os: String,
    kernel: String,
    uptime_secs: Option<u64>,
}

struct CpuInfo {
    loadavg: Option<(f64, f64, f64)>,
    usage_pct: Option<f64>,
}

struct MemInfo {
    total_kb: u64,
    used_kb: u64,
    pct: f64,
}

struct DiskInfo {
    total_kb: u64,
    used_kb: u64,
    pct: f64,
}

struct UsersInfo {
    users: Vec<String>,
}

/// `print_usage()` prints help information to the console
///
fn print_usage() {
    println!(
        "\nUsage: rust-cli-sysinfo [OPTIONS]

Display a system information dashboard

Options:
  -d, --disk <path>           Disk mount to report disk usage for [default: /home]
  -c, --cpu-sample-rate <ms>  CPU sampling window in milliseconds [default: 250]
  -n, --no-clear              Skip clearing the terminal before output
  -o, --no-color              Disable ANSI color output
  -h, --help                  Show this help message and exit"
    );
}

/// `read_first_line()` reads the first line of a file at the given path
///
fn read_first_line(path: &str) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut line = String::new();
    io::BufReader::new(file).read_line(&mut line).ok()?;
    let len = line.trim_end().len();
    line.truncate(len);
    Some(line)
}

/// `collect_system()` gathers system information, including hostname, OS, kernel version, and uptime
///
fn collect_system() -> SystemInfo {
    let hostname = read_first_line("/proc/sys/kernel/hostname").unwrap_or_else(|| "unknown".into());
    let kernel = read_first_line("/proc/sys/kernel/osrelease").unwrap_or_else(|| "unknown".into());

    let os = (|| -> Option<String> {
        let file = fs::File::open("/etc/os-release").ok()?;
        for line in io::BufReader::new(file).lines().map_while(Result::ok) {
            if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
                return Some(val.trim_matches('"').to_string());
            }
        }
        None
    })()
    .unwrap_or_else(|| "Unknown Linux".into());

    let uptime_secs = read_first_line("/proc/uptime").and_then(|raw| {
        raw.split_whitespace()
            .next()?
            .split('.')
            .next()?
            .parse()
            .ok()
    });

    let cpu_model = collect_cpu_model();
    let gpu_model = collect_gpu_model();

    SystemInfo {
        hostname,
        cpu_model,
        gpu_model,
        os,
        kernel,
        uptime_secs,
    }
}

/// `collect_cpu()` gathers CPU information, including load average and usage percentage
///
fn collect_cpu(sample_ms: u64) -> CpuInfo {
    struct CpuSnap {
        total: u64,
        idle: u64,
    }

    let read_snap = || -> Option<CpuSnap> {
        let line = read_first_line("/proc/stat")?;
        let mut fields = line
            .split_whitespace()
            .skip(1)
            .filter_map(|v| v.parse::<u64>().ok());

        let user = fields.next().unwrap_or(0);
        let nice = fields.next().unwrap_or(0);
        let system = fields.next().unwrap_or(0);
        let idle = fields.next().unwrap_or(0);
        let iowait = fields.next().unwrap_or(0);
        let irq = fields.next().unwrap_or(0);
        let softirq = fields.next().unwrap_or(0);
        let steal = fields.next().unwrap_or(0);

        let total = user + nice + system + idle + iowait + irq + softirq + steal;
        Some(CpuSnap {
            total,
            idle: idle + iowait,
        })
    };

    let loadavg = (|| -> Option<(f64, f64, f64)> {
        let line = read_first_line("/proc/loadavg")?;
        let mut parts = line.split_whitespace();
        Some((
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        ))
    })();

    let usage_pct = (|| -> Option<f64> {
        let snap1 = read_snap()?;

        thread::sleep(Duration::from_millis(sample_ms));
        let snap2 = read_snap()?;

        let d_total = snap2.total.saturating_sub(snap1.total);
        let d_idle = snap2.idle.saturating_sub(snap1.idle);

        if d_total == 0 {
            return Some(0.0);
        }

        let d_used = d_total.saturating_sub(d_idle);

        #[allow(clippy::cast_precision_loss)]
        let pct = (d_used as f64 / d_total as f64) * 100.0;
        Some(pct)
    })();

    CpuInfo { loadavg, usage_pct }
}

/// `collect_cpu_model()` attempts to read the CPU model name from /proc/cpuinfo
///
fn collect_cpu_model() -> Option<String> {
    // Read the standard Linux CPU info virtual file
    let file = File::open("/proc/cpuinfo").ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        // Look for the line containing the model name
        if line.starts_with("model name") {
            // Split by the colon and clean up the whitespace
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 1 {
                return Some(parts[1].trim().to_string());
            }
        }
    }
    None
}

/// `collect_gpu_model()` attempts to identify GPU models by reading from /sys/class/drm and using
/// the PCI IDs
///
fn collect_gpu_model() -> Vec<String> {
    let mut names = BTreeSet::new();

    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return Vec::new();
    };

    // Iterate over entries in the DRM directory, looking for card devices
    for entry in entries.flatten() {
        let path = entry.path();

        let Some(card_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // Ignore connector entries
        if !card_name.starts_with("card") || card_name.contains('-') {
            continue;
        }

        if let Some(name) = gpu_name_from_card(&path) {
            names.insert(name);
        }
    }

    names.into_iter().collect()
}

/// `gpu_name_from_card()` reads the vendor and device IDs from a given card path and looks up the
/// corresponding GPU name
///
fn gpu_name_from_card(card_path: &Path) -> Option<String> {
    let device_path = card_path.join("device");

    let vendor_id = read_hex_u16(&device_path.join("vendor"))?;
    let device_id = read_hex_u16(&device_path.join("device"))?;

    let vendor = Vendor::from_id(vendor_id)?;

    Device::from_vid_pid(vendor_id, device_id)
        .map(|device| format!("{} {}", vendor.name(), device.name()))
        .or_else(|| Some(vendor.name().to_owned()))
}

/// `read_hex_u16()` reads a hexadecimal value from a file and returns it as a u16
///
fn read_hex_u16(path: &Path) -> Option<u16> {
    let contents = fs::read_to_string(path).ok()?;
    let value = contents.trim().trim_start_matches("0x");

    u16::from_str_radix(value, 16).ok()
}

/// `collect_memory()` gathers memory information, including total, used, and percentage used
///
fn collect_memory() -> MemInfo {
    let mut total_kb = 0;
    let mut available_kb = None;
    let mut free_kb = 0;

    let parse_kb = |line: &str| -> u64 {
        line.split_whitespace()
            .nth(1)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };

    if let Ok(file) = fs::File::open("/proc/meminfo") {
        for line in io::BufReader::new(file).lines().map_while(Result::ok) {
            if line.starts_with("MemTotal:") {
                total_kb = parse_kb(&line);
            } else if line.starts_with("MemAvailable:") {
                available_kb = Some(parse_kb(&line));
            } else if line.starts_with("MemFree:") {
                free_kb = parse_kb(&line);
            }

            if total_kb > 0 && available_kb.is_some() && free_kb > 0 {
                break;
            }
        }
    }

    let avail_kb = available_kb.unwrap_or(free_kb);
    let used_kb = total_kb.saturating_sub(avail_kb);

    let pct = if total_kb > 0 {
        #[allow(clippy::cast_precision_loss)]
        let val = (used_kb as f64 / total_kb as f64) * 100.0;
        val
    } else {
        0.0
    };

    MemInfo {
        total_kb,
        used_kb,
        pct,
    }
}

/// `collect_disk()` gathers disk information, including total, used, and percentage used for the
/// given mount point
///
fn collect_disk(mount: &str) -> DiskInfo {
    let fallback = DiskInfo {
        total_kb: 0,
        used_kb: 0,
        pct: 0.0,
    };

    let Ok(output) = process::Command::new("df").args(["-kP", mount]).output() else {
        return fallback;
    };
    if !output.status.success() {
        return fallback;
    }

    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("");
    let Some(line) = stdout.lines().nth(1) else {
        return fallback;
    };

    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 6 {
        return fallback;
    }

    let total_kb: u64 = cols[1].parse().unwrap_or(0);
    let used_kb: u64 = cols[2].parse().unwrap_or(0);

    let pct = if total_kb > 0 {
        #[allow(clippy::cast_precision_loss)]
        let val = (used_kb as f64 / total_kb as f64) * 100.0;
        val
    } else {
        0.0
    };

    DiskInfo {
        total_kb,
        used_kb,
        pct,
    }
}

/// `collect_users()` gathers information about currently logged-in users by parsing the output of
/// the `who` command
///
fn collect_users() -> UsersInfo {
    let Ok(output) = process::Command::new("who").output() else {
        return UsersInfo { users: vec![] };
    };

    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("");
    let mut users: Vec<String> = stdout
        .lines()
        .filter_map(|l| l.split_whitespace().next().map(String::from))
        .collect();

    users.sort_unstable();
    users.dedup();

    UsersInfo { users }
}

/// `print_row()` prints a single row of the dashboard with appropriate coloring based on thresholds
///
fn print_row(label: &str, value: &str, threshold: &Threshold, c: &Colors) {
    let color = match threshold {
        Threshold::Check {
            value: v,
            warn: _,
            crit,
        } if v >= crit => c.red,
        Threshold::Check {
            value: v,
            warn,
            crit: _,
        } if v >= warn => c.yellow,
        Threshold::Check { .. } => c.green,
        Threshold::None => c.reset,
    };
    println!("{:<16} {}{}{}", label, color, value, c.reset);
}

/// `format_uptime()` converts uptime in seconds to a human-readable format ("001d:02h:03m:04s")
///
fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    format!("{days:03}d:{hours:02}h:{mins:02}m:{secs:02}s")
}

/// `format_size()` converts a size in kilobytes to a human-readable format ("100K")
///
fn format_size(kb: u64) -> String {
    #[allow(clippy::cast_precision_loss)]
    let k = kb as f64;

    if k >= 1_073_741_824.0 {
        format!("{:.1}T", k / 1_073_741_824.0)
    } else if k >= 1_048_576.0 {
        format!("{:.1}G", k / 1_048_576.0)
    } else if k >= 1_024.0 {
        format!("{:.1}M", k / 1_024.0)
    } else {
        format!("{kb}K")
    }
}

/// `render_system_section()` renders the static system information part of the dashboard
///
fn render_system_section(sys: &SystemInfo, c: &Colors) {
    print_row("  Hostname:", &sys.hostname, &Threshold::None, c);

    print_row(
        "  CPU:",
        sys.cpu_model.as_deref().unwrap_or("Unknown"),
        &Threshold::None,
        c,
    );

    let gpu_str = if sys.gpu_model.is_empty() {
        "Unknown".to_string()
    } else {
        sys.gpu_model.join("\n                 ")
    };

    print_row("  GPU(s):", &gpu_str, &Threshold::None, c);
    print_row("  Kernel:", &sys.kernel, &Threshold::None, c);

    let uptime_str = sys
        .uptime_secs
        .map_or_else(|| "unknown".to_string(), format_uptime);

    print_row("  Uptime:", &uptime_str, &Threshold::None, c);
}

/// `render_cpu_rows()` helper to render load average and CPU usage rows
fn render_cpu_rows(cpu: &CpuInfo, c: &Colors) {
    let load_str = cpu.loadavg.map_or_else(
        || "N/A".to_string(),
        |(l1, l5, l15)| format!("{l1:.2}, {l5:.2}, {l15:.2} (1m, 5m, 15m)"),
    );
    print_row("  Load averages:", &load_str, &Threshold::None, c);

    let (cpu_str, cpu_thresh) = cpu.usage_pct.map_or_else(
        || ("N/A".to_string(), Threshold::None),
        |v| {
            (
                format!("{v:.1}%"),
                Threshold::Check {
                    value: v,
                    warn: 70.0,
                    crit: 90.0,
                },
            )
        },
    );
    print_row("  CPU usage:", &cpu_str, &cpu_thresh, c);
}

/// `render_mem_row()` helper to render the memory usage row
fn render_mem_row(mem: &MemInfo, c: &Colors) {
    let mem_str = format!(
        "{:.1}% ({}MB/{}MB)",
        mem.pct,
        mem.used_kb / 1024,
        mem.total_kb / 1024
    );
    print_row(
        "  Memory usage:",
        &mem_str,
        &Threshold::Check {
            value: mem.pct,
            warn: 75.0,
            crit: 90.0,
        },
        c,
    );
}

/// `render_disk_row()` helper to render the disk usage row
fn render_disk_row(disk: &DiskInfo, mount: &str, c: &Colors) {
    let (disk_str, disk_thresh) = if disk.total_kb == 0 {
        ("N/A".to_string(), Threshold::None)
    } else {
        let text = format!(
            "{:.1}% ({}/{}) of {}",
            disk.pct,
            format_size(disk.used_kb),
            format_size(disk.total_kb),
            mount
        );
        (
            text,
            Threshold::Check {
                value: disk.pct,
                warn: 80.0,
                crit: 95.0,
            },
        )
    };
    print_row("  Disk usage:", &disk_str, &disk_thresh, c);
}

/// `render_stats_section()` renders the dynamic resource metrics part of the dashboard
///
fn render_stats_section(cpu: &CpuInfo, mem: &MemInfo, disk: &DiskInfo, mount: &str, c: &Colors) {
    render_cpu_rows(cpu, c);
    render_mem_row(mem, c);
    render_disk_row(disk, mount, c);
}

/// `render_user_section()` renders the logged-in user information part of the dashboard
///   
fn render_user_section(users: &UsersInfo, c: &Colors) {
    let user_str = if users.users.is_empty() {
        "Nobody".to_string()
    } else {
        users.users.join(", ")
    };

    print_row("  User(s):", &user_str, &Threshold::None, c);
}

/// `render_dashboard()` takes all collected information and renders the dashboard to the console
///
fn render_dashboard(
    sys: &SystemInfo,
    cpu: &CpuInfo,
    mem: &MemInfo,
    disk: &DiskInfo,
    users: &UsersInfo,
    opts: &Opts,
    c: &Colors,
) {
    if opts.clear {
        print!("\x1bc");
    }

    println!("\n  {}{}Rust-CLI-SysInfo{}", c.bold, c.cyan, c.reset);
    println!("  {}{}{}", c.cyan, sys.os, c.reset);
    println!("  {}{}{}", c.cyan, SEP, c.reset);

    render_system_section(sys, c);
    render_stats_section(cpu, mem, disk, &opts.disk_mount, c);
    render_user_section(users, c);

    println!("  {}{}{}", c.cyan, SEP, c.reset);
    println!("{}{}  v{}{}", c.bold, c.cyan, VERSION, c.reset);
}

/// `main()` makes everything go!
///
fn main() {
    // Parse command-line options and initialize colors
    let opts = Opts::from_args();
    let colors = Colors::new(opts.color);

    // Gather system information
    let sys = collect_system();
    let cpu = collect_cpu(opts.cpu_sample_ms);
    let mem = collect_memory();
    let disk = collect_disk(&opts.disk_mount);
    let users = collect_users();

    // Render the dashboard to the console
    render_dashboard(&sys, &cpu, &mem, &disk, &users, &opts, &colors);
}
