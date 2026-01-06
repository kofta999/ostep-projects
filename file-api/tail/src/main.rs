use anyhow::Context;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{Whence, close, lseek, read};
use std::io::{self, Write};

const DEFAULT_COUNT: i32 = 10;
const BUFFER_SIZE: u16 = 4096;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1);

    let mut target_count = DEFAULT_COUNT;
    let mut filename = String::new();

    for arg in args {
        if let Some(count) = arg.strip_prefix('-') {
            target_count = count.parse::<i32>().context("n must be a number")?;
        } else {
            filename = arg;
        }
    }

    if filename.is_empty() {
        anyhow::bail!("Usage: tail -n <file>");
    }

    let fd = open(filename.as_str(), OFlag::O_RDONLY, Mode::S_IRUSR)?;

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

    lseek(&fd, pos, Whence::SeekSet)?;
    loop {
        let bytes_read = read(&fd, &mut buf).unwrap();
        if bytes_read == 0 {
            break;
        }
        io::stdout().write_all(&buf[..bytes_read])?;
    }

    close(fd)?;

    Ok(())
}
