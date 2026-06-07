# Rust CLI Sysinfo

A simple Rust-based system services dashboard that runs in a Linux terminal

[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/richbl/rust-cli-sysinfo)](https://rust-reportcard.xuri.me/report/github.com/richbl/rust-cli-sysinfo)
![GitHub Release](https://img.shields.io/github/v/release/richbl/rust-cli-sysinfo?include_prereleases&sort=semver&display_name=tag&color=blue)

<!-- markdownlint-disable MD033 -->
<p align="center">
<img width="650" alt="Screenshot showing cycling trainer" src="https://raw.githubusercontent.com/richbl/rust-cli-sysinfo/refs/heads/main/.github/assets/rust-cli-sysinfo_output.png">
</p>
<!-- markdownlint-enable MD033 -->

## Rationale

The goal of **rust-cli-sysinfo** is to create a simple and efficient terminal-based dashboard for presenting the status of various system services on Linux. It is designed to provide a quick overview of the status of various services, allowing users to easily identify any issues or bottlenecks in their system.

## Requirements

**rust-cli-sysinfo** is designed to run natively on Linux systems only, with no other requirements.

However, since **rust-cli-sysinfo** is built using Rust, if you want to build this application from project sources, you'll need to have Rust installed on your system.

## Installation

Simply copy the binary from the latest release to your system (e.g., `/usr/local/bin` or `/home/[user]/.local/bin`) and run it. You can find the latest release on the [GitHub releases page](https://github.com/richbl/rust-cli-sysinfo/releases).

> If installing into `/home/[user]/.local/bin`, make sure to add that directory to your `PATH` environment variable, if it's not already included:

```bash
export PATH="$PATH:/home/[user]/.local/bin"
```

In my own case, I've added a call to `rust-cli-sysinfo` in my `.bashrc` file, so it runs every time I open a new terminal session.

### Building the Project from Source

To build the project, run the following command in the root directory of the project:

```bash
cargo build --release --bin rust-cli-sysinfo
```

Then, copy the resulting binary from `target/release/rust-cli-sysinfo` to your desired location (e.g., `/usr/local/bin` or `/home/[user]/.local/bin`).

That's it. You can now run the application by executing `rust-cli-sysinfo` in your terminal.

## Roadmap

At the moment, there's not much of a roadmap to consider. I may make the workflow logic a bit more modular so it'd be easier to add new services to monitor. But for now, **rust-cli-sysinfo** just works and does what it's supposed to do: it checks the status of a few services and displays the results in a nice format.

In general, this is pretty simple executable doing some pretty basic stuff. But if you have any thoughts or ideas for improvement, send them my way.
