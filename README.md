# Progress

A simple command-line utility to monitor the progress of data being piped through it. It reads from standard input and writes to standard output, while printing progress information to standard error.

## Description

`progress` is a tool that shows you how much data is passing through a pipe, how long it has taken, and the current speed. This is useful when you have a long-running command and want to see if it's still working and how fast it's going.

The progress information is written to standard error, so it doesn't interfere with the data being piped from standard output to another command.

## Usage

You can pipe the output of any command into `progress`.

### Example

To monitor the progress of gzipping a large file:

```sh
cat /dev/urandom | head -c 1G | progress | gzip > random.gz
```

This will show progress information on your terminal while `random.gz` is being created.

## Building

To build the project, you need to have Rust and Cargo installed. You can find instructions on how to install them at [rust-lang.org](https://www.rust-lang.org/).

Once you have Rust and Cargo installed, you can build the project with the following command:

```sh
cargo build
```

For an optimized release build, use:

```sh
cargo build --release
```

The executable will be located in `target/debug/progress` or `target/release/progress`.

## Testing

To run the tests, use the following command:

```sh
cargo test
```
