# Rust CLI Sysinfo (RCS)

**Rust-CLI-Sysinfo** (RCS) is a simple Rust-based system services utility that runs in a Linux terminal, offering a modular and customizable way to display the status of various system services.

[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/richbl/rust-cli-sysinfo)](https://rust-reportcard.xuri.me/report/github.com/richbl/rust-cli-sysinfo)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/e79c70051570426bb353b230332fe623)](https://app.codacy.com/gh/richbl/rust-cli-sysinfo/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
![GitHub Release](https://img.shields.io/github/v/release/richbl/rust-cli-sysinfo?include_prereleases&sort=semver&display_name=tag&color=blue)

## RCS Screenshots

### The Default RCS Output

The default output of the `rust-cli-sysinfo` utility is shown below:

<!-- markdownlint-disable MD033 -->
<p align="center">
<img width="650" alt="Screenshot showing Rust CLI Sysinfo output" src="https://raw.githubusercontent.com/richbl/rust-cli-sysinfo/refs/heads/main/.github/assets/rust-cli-sysinfo_output.png">
</p>
<!-- markdownlint-enable MD033 -->

### The RCS Help Output

Calling `rust-cli-sysinfo -h` (or `rust-cli-sysinfo --help`) shows the following help output:

<!-- markdownlint-disable MD033 -->
<p align="center">
<img width="650" alt="Screenshot showing Rust CLI Sysinfo output" src="https://raw.githubusercontent.com/richbl/rust-cli-sysinfo/refs/heads/main/.github/assets/rust-cli-sysinfo_help.png">
</p>
<!-- markdownlint-enable MD033 -->

### Customizing RCS with Services Tokens

One of the more useful features of **Rust-CLI-Sysinfo** is the ability to configure any of the available services tokens. The output of the services tokens display called with `rust-cli-sysinfo -s` (or `rust-cli-sysinfo --services`)is shown below:

<!-- markdownlint-disable MD033 -->
<p align="center">
<img width="650" alt="Screenshot showing Rust CLI Sysinfo output" src="https://raw.githubusercontent.com/richbl/rust-cli-sysinfo/refs/heads/main/.github/assets/rust-cli-sysinfo_tokens.png">
</p>
<!-- markdownlint-enable MD033 -->

#### A Customized RCS Output

The following screenshot shows the output of **Rust-CLI-Sysinfo** when configured with services tokens. In the screenshot below, **Rust-CLI-Sysinfo** has been configured logged-in users, OS name/version, memory usage, disk usage, CPU details, and GPU(s) details.

The command used to generate this output is `rust-cli-sysinfo -s USR-OS-RAM-DSK-CPUM-GPU`.

<!-- markdownlint-disable MD033 -->
<p align="center">
<img width="650" alt="Screenshot showing Rust CLI Sysinfo output" src="https://raw.githubusercontent.com/richbl/rust-cli-sysinfo/refs/heads/main/.github/assets/rust-cli-sysinfo_tokens_output.png">
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

- Configurable services tokens, allowing users to choose which services to display, and the order in which they should be displayed

- Color status indicators for CPU, memory, and disk usage, with threshold levels represented by:
    - Normal (green)
    - Warning (yellow)
    - Critical (red)

- Display configuration options, including:
    - `--no-color`: Disable colored status indicators in the output
    - `--no-clear`: Disable clearing the screen before running the utility

- Designed explicitly without the need for an external configuration file: configure everything with command line flags

- This is a Linux-only utility, designed to be lightweight and efficient, relying on native system calls and libraries
    - No external dependencies

- Built using Rust, ensuring high performance and reliability

## Rationale

- The goal of **Rust-CLI-Sysinfo** is to create a simple, efficient, and modular terminal-based utility for presenting the status of various system services in Linux.

## Requirements

- **Rust-CLI-Sysinfo** is designed to run natively on Linux systems, with no other requirements

  However, since **Rust-CLI-Sysinfo** is built using Rust, if you want to build this application from project sources, you'll need to have Rust installed on your system.

## Installation

Simply copy the binary from the latest release to your system (e.g., `/usr/local/bin` or `/home/[user]/.local/bin`). You can find the latest release on the [GitHub releases page](https://github.com/richbl/rust-cli-sysinfo/releases).

> Note: If installing into `/home/[user]/.local/bin`, make sure to add that directory to your `PATH` environment variable if it's not already added.

### Building the RCS Project from Sources

1. To build the project, clone the project onto your system and run the following command in the root directory of the project:

    ```console
    cargo build --release --bin rust-cli-sysinfo
    ```

2. Then, copy the resulting binary from `target/release/rust-cli-sysinfo` to your desired location (e.g., `/usr/local/bin` or `/home/[user]/.local/bin`).

That's it. You can now run the application by executing `rust-cli-sysinfo` in your terminal.

## Usage

- To run the utility, simply execute `rust-cli-sysinfo` in your terminal. By default, the utility will clear the terminal and display the status of all available system services.

- To get help information about the available command-line options, run `rust-cli-sysinfo -h` (or `rust-cli-sysinfo --help`)

### Running in a New Terminal Session

I've added a call to `rust-cli-sysinfo` in my `.bashrc` file, so it runs every time I open a new terminal session.

> This is precisely the use case that I had in mind when I created this project.

## Roadmap

In general, this is pretty simple executable doing some pretty basic stuff. But if you have any thoughts or ideas for improvement, send them my way.
