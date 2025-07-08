use std::io::{self, Read, Write};
use std::time::{Duration, Instant};


fn main() -> io::Result<()> {
    let mut buffer = [0; 8192];
    let mut total_bytes = 0;
    let start_time = Instant::now();
    let mut last_update = Instant::now();

    loop {
        let bytes_read = io::stdin().read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        io::stdout().write_all(&buffer[..bytes_read])?;
        total_bytes += bytes_read;

        let elapsed = start_time.elapsed();
        let speed = if elapsed.as_secs() > 0 {
            total_bytes as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        if last_update.elapsed() >= Duration::from_secs(1) {
            eprint!(
                "\rBytes: {}, Time: {:.2}s, Speed: {:.2} B/s",
                total_bytes,
                elapsed.as_secs_f64(),
                speed
            );
            io::stderr().flush()?;
            last_update = Instant::now();
        }
    }

    let elapsed = start_time.elapsed();
    let speed = if elapsed.as_secs() > 0 {
        total_bytes as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };
    eprint!(
        "\rBytes: {}, Time: {:.2}s, Speed: {:.2} B/s",
        total_bytes,
        elapsed.as_secs_f64(),
        speed
    );
    io::stderr().flush()?;

    eprintln!();
    Ok(())
}