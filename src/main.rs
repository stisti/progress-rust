use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        // >= 1 GB
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        // >= 1 MB
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        // >= 1 KB
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        // < 1 KB
        format!("{} B", bytes)
    }
}

fn format_speed(bytes_per_second: f64) -> String {
    if bytes_per_second >= 1_073_741_824.0 {
        // >= 1 GB/s
        format!("{:.2} GB/s", bytes_per_second / 1_073_741_824.0)
    } else if bytes_per_second >= 1_048_576.0 {
        // >= 1 MB/s
        format!("{:.2} MB/s", bytes_per_second / 1_048_576.0)
    } else if bytes_per_second >= 1024.0 {
        // >= 1 KB/s
        format!("{:.2} KB/s", bytes_per_second / 1024.0)
    } else {
        // < 1 KB/s
        format!("{:.2} B/s", bytes_per_second)
    }
}

pub fn run<R: Read, W1: Write, W2: Write + Send + 'static>(
    mut reader: R,
    mut writer: W1,
    mut err_writer: W2,
) -> io::Result<W2> {
    let total_bytes = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let start_time = Instant::now();

    let total_bytes_clone = Arc::clone(&total_bytes);
    let done_clone = Arc::clone(&done);

    let display_thread = thread::spawn(move || {
        while !done_clone.load(Ordering::Relaxed) {
            let elapsed = start_time.elapsed();
            let total_bytes_val = total_bytes_clone.load(Ordering::Relaxed);
            let speed = if elapsed.as_secs() > 0 {
                total_bytes_val as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };

            write!(
                &mut err_writer,
                "\rBytes: {}, Time: {:.2}s, Speed: {}",
                format_bytes(total_bytes_val),
                elapsed.as_secs_f64(),
                format_speed(speed)
            )
            .unwrap();
            err_writer.flush().unwrap();

            thread::sleep(Duration::from_secs(1));
        }

        // Final update after the loop is done.
        let elapsed = start_time.elapsed();
        let total_bytes_val = total_bytes_clone.load(Ordering::Relaxed);
        let speed = if elapsed.as_secs() > 0 {
            total_bytes_val as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        write!(
            &mut err_writer,
            "\rBytes: {}, Time: {:.2}s, Speed: {}",
            format_bytes(total_bytes_val),
            elapsed.as_secs_f64(),
            format_speed(speed)
        )
        .unwrap();
        err_writer.flush().unwrap();

        err_writer
    });

    let mut buffer = [0; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        writer.write_all(&buffer[..bytes_read])?;
        total_bytes.fetch_add(bytes_read as u64, Ordering::Relaxed);
    }

    done.store(true, Ordering::Relaxed);
    let mut err_writer = display_thread.join().unwrap();

    writeln!(&mut err_writer)?;
    Ok(err_writer)
}

fn main() -> io::Result<()> {
    run(io::stdin(), io::stdout(), io::stderr())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_run() {
        let input_data = "hello world";
        let reader = Cursor::new(input_data);
        let writer = Vec::new();
        let err_writer = Vec::new();

        let result = run(reader, writer, err_writer);

        assert!(result.is_ok());
        let err_writer = result.unwrap();

        let err_output = String::from_utf8(err_writer).unwrap();
        assert!(err_output.contains("\rBytes: 11 B,"));
        assert!(err_output.contains("Speed:"));
        assert!(err_output.ends_with("/s\n"));
    }

    #[test]
    fn test_run_empty_input() {
        let input_data = "";
        let reader = Cursor::new(input_data);
        let writer = Vec::new();
        let err_writer = Vec::new();

        let result = run(reader, writer, err_writer);

        assert!(result.is_ok());
        let err_writer = result.unwrap();

        let err_output = String::from_utf8(err_writer).unwrap();
        assert!(err_output.contains("\rBytes: 0 B,"));
        assert!(err_output.contains("Speed:"));
        assert!(err_output.ends_with("/s\n"));
    }

    #[test]
    fn test_format_speed_bytes() {
        assert_eq!(format_speed(0.0), "0.00 B/s");
        assert_eq!(format_speed(512.5), "512.50 B/s");
        assert_eq!(format_speed(1023.0), "1023.00 B/s");
    }

    #[test]
    fn test_format_speed_kilobytes() {
        assert_eq!(format_speed(1024.0), "1.00 KB/s");
        assert_eq!(format_speed(1536.0), "1.50 KB/s");
        assert_eq!(format_speed(1048575.0), "1024.00 KB/s");
    }

    #[test]
    fn test_format_speed_megabytes() {
        assert_eq!(format_speed(1048576.0), "1.00 MB/s");
        assert_eq!(format_speed(1572864.0), "1.50 MB/s");
        assert_eq!(format_speed(1073741823.0), "1024.00 MB/s");
    }

    #[test]
    fn test_format_speed_gigabytes() {
        assert_eq!(format_speed(1073741824.0), "1.00 GB/s");
        assert_eq!(format_speed(1610612736.0), "1.50 GB/s");
        assert_eq!(format_speed(10737418240.0), "10.00 GB/s");
    }

    #[test]
    fn test_format_bytes_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn test_format_bytes_kilobytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048575), "1024.00 KB");
    }

    #[test]
    fn test_format_bytes_megabytes() {
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1572864), "1.50 MB");
        assert_eq!(format_bytes(1073741823), "1024.00 MB");
    }

    #[test]
    fn test_format_bytes_gigabytes() {
        assert_eq!(format_bytes(1073741824), "1.00 GB");
        assert_eq!(format_bytes(1610612736), "1.50 GB");
        assert_eq!(format_bytes(10737418240), "10.00 GB");
    }
}
