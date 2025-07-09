use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

pub fn run<R: Read, W1: Write, W2: Write>(
    mut reader: R,
    mut writer: W1,
    mut err_writer: W2,
) -> io::Result<()> {
    let mut buffer = [0; 8192];
    let mut total_bytes = 0;
    let start_time = Instant::now();
    let mut last_update = Instant::now();

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        writer.write_all(&buffer[..bytes_read])?;
        total_bytes += bytes_read;

        let elapsed = start_time.elapsed();
        let speed = if elapsed.as_secs() > 0 {
            total_bytes as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        if last_update.elapsed() >= Duration::from_secs(1) {
            write!(
                err_writer,
                "\rBytes: {}, Time: {:.2}s, Speed: {:.2} B/s",
                total_bytes,
                elapsed.as_secs_f64(),
                speed
            )?;
            err_writer.flush()?;
            last_update = Instant::now();
        }
    }

    let elapsed = start_time.elapsed();
    let speed = if elapsed.as_secs() > 0 {
        total_bytes as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };
    write!(
        err_writer,
        "\rBytes: {}, Time: {:.2}s, Speed: {:.2} B/s",
        total_bytes,
        elapsed.as_secs_f64(),
        speed
    )?;
    err_writer.flush()?;

    writeln!(err_writer)?;
    Ok(())
}

fn main() -> io::Result<()> {
    run(io::stdin(), io::stdout(), io::stderr())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_run() {
        let input_data = "hello world";
        let mut reader = Cursor::new(input_data);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        let result = run(&mut reader, &mut writer, &mut err_writer);

        assert!(result.is_ok());
        assert_eq!(writer, input_data.as_bytes());

        let err_output = String::from_utf8(err_writer).unwrap();
        assert!(err_output.starts_with("\rBytes: 11,"));
        assert!(err_output.ends_with("B/s\n"));
    }

    #[test]
    fn test_run_empty_input() {
        let input_data = "";
        let mut reader = Cursor::new(input_data);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        let result = run(&mut reader, &mut writer, &mut err_writer);

        assert!(result.is_ok());
        assert!(writer.is_empty());

        let err_output = String::from_utf8(err_writer).unwrap();
        assert!(err_output.starts_with("\rBytes: 0,"));
        assert!(err_output.ends_with("B/s\n"));
    }
}