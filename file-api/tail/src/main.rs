use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{Whence, close, lseek, read};
use std::io::{self, Write};

const DEFAULT_COUNT: i32 = 10;
const BUFFER_SIZE: u16 = 4096;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err("Missing arguments".into());
    }
    let mut target_count: Option<i32> = None;

    if args[1].starts_with("-") {
        let temp = args[1][1..]
            .parse::<i32>()
            .map_err(|_| "Count should be a number")?;

        target_count = Some(temp);

        if args.len() != 3 {
            return Err("Filename required".into());
        }
    }

    let filename = if target_count.is_some() {
        &args[2]
    } else {
        &args[1]
    }
    .as_str();

    let mut target_count = target_count.unwrap_or(DEFAULT_COUNT);

    if let Ok(fd) = open(filename, OFlag::O_RDONLY, Mode::S_IRUSR) {
        let file_size = lseek(&fd, 0, Whence::SeekEnd)
            .map_err(|_| "Error while seeking to the end of the file")?;
        let mut pos = file_size;
        let mut buf = [0u8; BUFFER_SIZE as usize];

        while target_count >= 0 && pos > 0 {
            let to_read = std::cmp::min(pos, BUFFER_SIZE as i64);
            pos -= to_read;

            lseek(&fd, pos, Whence::SeekSet).map_err(|_| "Could not seek to position")?;
            read(&fd, &mut buf[..to_read as usize]).map_err(|_| "Could not read to the buffer")?;

            for (i, &byte) in buf[..to_read as usize].iter().enumerate().rev() {
                if byte == b'\n' {
                    target_count -= 1;
                    if target_count < 0 {
                        pos += i as i64 + 1;
                        break;
                    }
                }
            }
        }

        lseek(&fd, pos, Whence::SeekSet);
        loop {
            let bytes_read = read(&fd, &mut buf).unwrap();
            if bytes_read == 0 {
                break;
            }
            io::stdout().write_all(&buf[..bytes_read]);
        }

        close(fd).map_err(|_| "Could not close the file")?;
    }

    Ok(())
}
