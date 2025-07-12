use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

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
                "\rBytes: {}, Time: {:.2}s, Speed: {:.2} B/s",
                total_bytes_val,
                elapsed.as_secs_f64(),
                speed
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
            "\rBytes: {}, Time: {:.2}s, Speed: {:.2} B/s",
            total_bytes_val,
            elapsed.as_secs_f64(),
            speed
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
        assert!(err_output.contains("\rBytes: 11,"));
        assert!(err_output.ends_with("B/s\n"));
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
        assert!(err_output.contains("\rBytes: 0,"));
        assert!(err_output.ends_with("B/s\n"));
    }
}
