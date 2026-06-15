# Rust CLI Sysinfo (RCS)

**Rust-CLI-Sysinfo** (RCS) is a simple Rust-based system services utility that runs in a Linux terminal, offering a modular and customizable way to display the status of various system services.

![GitHub Release](https://img.shields.io/github/v/release/richbl/rust-cli-sysinfo?include_prereleases&sort=semver&display_name=tag&color=blue)
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/richbl/rust-cli-sysinfo)](https://rust-reportcard.xuri.me/report/github.com/richbl/rust-cli-sysinfo)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/e79c70051570426bb353b230332fe623)](https://app.codacy.com/gh/richbl/rust-cli-sysinfo/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
<!-- markdownlint-disable MD033 -->
<a href="https://codeberg.org/richbl/rust-cli-sysinfo">
  <img alt="Link to Codeberg Mirror" src="https://img.shields.io/badge/Codeberg%20Mirror-Rust--CLI--SysInfo-red?style=flat&logo=codeberg&logoColor=white&labelColor=grey&color=blue&link=https%3A%2F%2Fcodeberg.org%2Frichbl%2Frust-cli-sysinfo">
</a>
<!-- markdownlint-enable MD033 -->

## RCS Screenshots

### The Default RCS Output

The default output of the `rust-cli-sysinfo` utility is shown below. Notice that several of the services are (optionally) displayed in green to indicate their status:

<!-- markdownlint-disable MD033 -->
<p align="center">
<img width="" alt="Screenshot showing Rust CLI Sysinfo output" src="https://raw.githubusercontent.com/richbl/rust-cli-sysinfo/refs/heads/main/.github/assets/rust-cli-sysinfo_output.png">
</p>
<!-- markdownlint-enable MD033 -->

### The RCS Help Output

Calling `rust-cli-sysinfo -h` (or `rust-cli-sysinfo --help`) displays the following help output:

<!-- markdownlint-disable MD033 -->
<p align="center">
<img width="" alt="Screenshot showing Rust CLI Sysinfo output" src="https://raw.githubusercontent.com/richbl/rust-cli-sysinfo/refs/heads/main/.github/assets/rust-cli-sysinfo_help.png">
</p>
<!-- markdownlint-enable MD033 -->

### Customizing RCS with Service Tokens

One of the more useful features of **Rust-CLI-Sysinfo** is the ability to configure any of the available services using service tokens. The output of the service tokens display called with `rust-cli-sysinfo -s` (or `rust-cli-sysinfo --services`) is shown below:

<!-- markdownlint-disable MD033 -->
<p align="center">
<img width="" alt="Screenshot showing Rust CLI Sysinfo output" src="https://raw.githubusercontent.com/richbl/rust-cli-sysinfo/refs/heads/main/.github/assets/rust-cli-sysinfo_tokens.png">
</p>
<!-- markdownlint-enable MD033 -->

#### A Customized RCS Output

The following screenshot shows the output of **Rust-CLI-Sysinfo** when configured with service tokens. In the screenshot below, **Rust-CLI-Sysinfo** has been configured to display only the following services:

- Users currently logged into the system
- OS name/version
- Memory usage
- Disk usage
- CPU details
- GPU(s) details

The command used to generate this output is `rust-cli-sysinfo -s USR-OS-RAMU-DSKU-CPU-GPU`.

<!-- markdownlint-disable MD033 -->
<p align="center">
<img width="" alt="Screenshot showing Rust CLI Sysinfo output" src="https://raw.githubusercontent.com/richbl/rust-cli-sysinfo/refs/heads/main/.github/assets/rust-cli-sysinfo_tokens_output.png">
</p>
<!-- markdownlint-enable MD033 -->

## RCS Features

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
    - Disk usage (defaults to `/home`, but can be configured with the `-d/--disk` flag)
    - Users currently logged into the system

- Configurable service tokens, allowing users to choose which services to display, and the order in which they should be displayed

- Color status indicators for CPU, memory, and disk usage, with threshold levels represented by:
    - Normal (green)
    - Warning (yellow)
    - Critical (red)

- Display configuration options, including:
    - `--no-color`: disable colored status indicators in the output
    - `--no-clear`: disable clearing the screen before running the utility

- Developed explicitly without the need for an external configuration file: configure everything with command line flags

- This is a Linux-only utility, designed to be lightweight and efficient, relying on native system calls and libraries
    - No external dependencies

- Built using Rust, ensuring high performance and reliability

## Rationale

- The goal of **Rust-CLI-Sysinfo** is to create a simple, efficient, and modular terminal-based utility for presenting the status of various system services in Linux

## Requirements

- **Rust-CLI-Sysinfo** is designed to run natively on Linux systems, with no other requirements

However, if you prefer to build **Rust-CLI-Sysinfo* from project sources, you'll need to have Rust installed on your system.

## Installation

Simply copy the `rust-cli-sysinfo` binary from the latest release to your system (e.g., `/usr/local/bin` or `/home/[user]/.local/bin`). You can find the latest release on the [GitHub releases page](https://github.com/richbl/rust-cli-sysinfo/releases).

### Building the RCS Project from Sources

1. To build the project, clone the project onto your system and run the following command in the root directory of the project:

    ```console
    cargo build --release --bin rust-cli-sysinfo
    ```

2. Copy the resulting binary from `target/release/rust-cli-sysinfo` to your desired location (e.g., `/usr/local/bin` or `/home/[user]/.local/bin`)

3. That's it. You can now run the application by executing `rust-cli-sysinfo` in your terminal

## Usage

- To run the utility, simply execute `rust-cli-sysinfo` in your terminal. By default, the utility will clear the terminal and display the status of all available system services.

- To get help information about the available command-line options, run `rust-cli-sysinfo -h` (or `rust-cli-sysinfo --help`)

### Running in a New Terminal Session

I've added a call to `rust-cli-sysinfo` in my `.bashrc` file, so it runs every time I open a new terminal window.

> This is precisely the use case that I had in mind when I created this project. It's a quick way to get an overview of the status of various system services without having to run various CLI commands or run separate system utilities.

## Roadmap

In general, this is a pretty simple executable doing some pretty basic stuff. But if you have any thoughts or ideas for improvements, send them my way.
