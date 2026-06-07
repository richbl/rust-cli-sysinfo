# Rust CLI Sysinfo

A simple Rust-based system services dashboard that runs in a Linux terminal

[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/richbl/rust-cli-sysinfo)](https://rust-reportcard.xuri.me/report/github.com/richbl/rust-cli-sysinfo)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/e79c70051570426bb353b230332fe623)](https://app.codacy.com/gh/richbl/rust-cli-sysinfo/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
![GitHub Release](https://img.shields.io./github/v/release/richbl/rust-cli-sysinfo?include_prereleases&sort=semver&display_name=tag&color=blue)

<!-- markdownlint-disable MD033 -->
<p align="center">
<img width="650" alt="Screenshot showing cycling trainer" src="https://raw.githubusercontent.com/richbl/rust-cli-sysinfo/refs/heads/main/.github/assets/rust-cli-sysinfo_output.png">
</p>
<!-- markdownlint-enable MD033 -->

## Features

- Displays the status of various system services, including:
    - Linux distribution name and version
    - Hostname
    - CPU details
    - GPU details
    - Kernel version
    - Uptime
    - System load average (over 1, 5, and 15 minutes)
    - CPU usage
    - Memory usage
    - Disk usage (defaults to `/home`, but can be overridden with the `-d/--disk` flag)
    - Users currently logged into the system

- Color indicators for CPU, memory, and disk usage, with thresholds for normal (green), warning (yellow) and critical (red) levels

- Dashboard display configuration options, including:
    - `--no-color`: Disable colored output
    - `--no-clear`: Disable clearing the screen before running the dashboard

- This is a Linux-only dashboard, designed to be lightweight and efficient, relying on native system calls and libraries

- Built using Rust, ensuring high performance and reliability

## Rationale

The goal of **rust-cli-sysinfo** is to create a simple and efficient terminal-based dashboard for presenting the status of various system services on Linux. It is designed to provide a quick overview of the status of various services, allowing users to easily identify any issues or bottlenecks in their system.

## Requirements

**rust-cli-sysinfo** is designed to run natively on Linux systems only, with no other requirements.

However, since **rust-cli-sysinfo** is built using Rust, if you want to build this application from project sources, you'll need to have Rust installed on your system.

## Installation

Simply copy the binary from the latest release to your system (e.g., `/usr/local/bin` or `/home/[user]/.local/bin`). You can find the latest release on the [GitHub releases page](https://github.com/richbl/rust-cli-sysinfo/releases).

If installing into `/home/[user]/.local/bin`, make sure to add that directory to your `PATH` environment variable, if it's not already included:

```console
export PATH="$PATH:/home/[user]/.local/bin"
```

### Building the Project from Source

To build the project, run the following command in the root directory of the project:

```console
cargo build --release --bin rust-cli-sysinfo
```

Then, copy the resulting binary from `target/release/rust-cli-sysinfo` to your desired location (e.g., `/usr/local/bin` or `/home/[user]/.local/bin`).

That's it. You can now run the application by executing `rust-cli-sysinfo` in your terminal.

## Usage

To run the dashboard, simply execute `rust-cli-sysinfo` in your terminal. By default, the dashboard will clear the terminal display the status of various system services.

To get help information about the available command-line options, run `rust-cli-sysinfo --help`:

```console
Usage: rust-cli-sysinfo [OPTIONS]

Display a system information dashboard

Options:
  -d, --disk <path>           Disk mount to report disk usage for [default: /home]
  -c, --cpu-sample-rate <ms>  CPU sampling window in milliseconds [default: 250]
  -n, --no-clear              Skip clearing the terminal before output
  -o, --no-color              Disable ANSI color output
  -h, --help                  Show this help message and exit
```

### Running Whenever You Start a New Terminal Session

I've added a call to `rust-cli-sysinfo` in my `.bashrc` file, so it runs every time I open a new terminal session, which is precisely the use case that I had initially when I created this project.

## Roadmap

After a brief stabilization phase, I hope to refactor the code a bit to make it more modular and easier to maintain. I also plan to add support for monitoring additional services in the future, as well as improving the overall user experience and adding some customization options.

In general, this is pretty simple executable doing some pretty basic stuff. But if you have any thoughts or ideas for improvement, send them my way.
