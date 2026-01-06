use anyhow::Context;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{Whence, close, lseek, read};
use std::io::{self, Write};

const DEFAULT_COUNT: i32 = 10;
const BUFFER_SIZE: u16 = 4096;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        anyhow::bail!("Usage: tail -n <file>");
    }
    let mut target_count: Option<i32> = None;

    if args[1].starts_with("-") {
        let temp = args[1][1..].parse::<i32>().context("n must be a number")?;

        target_count = Some(temp);

        if args.len() != 3 {
            anyhow::bail!("<file> required");
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
        let file_size = lseek(&fd, 0, Whence::SeekEnd)?;
        let mut pos = file_size;
        let mut buf = [0u8; BUFFER_SIZE as usize];

        while target_count >= 0 && pos > 0 {
            let to_read = std::cmp::min(pos, BUFFER_SIZE as i64);
            pos -= to_read;

            lseek(&fd, pos, Whence::SeekSet)?;
            read(&fd, &mut buf[..to_read as usize])?;

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

        close(fd)?;
    }

    Ok(())
}
