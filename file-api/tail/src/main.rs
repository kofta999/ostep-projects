use anyhow::Context;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{Whence, close, lseek, read};

const DEFAULT_COUNT: i32 = 10;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        anyhow::bail!("Usage: tail -n <file>");
    }
    let mut count: Option<i32> = None;

    if args[1].starts_with("-") {
        let temp = args[1][1..].parse::<i32>().context("n must be a number")?;

        count = Some(temp);

        if args.len() != 3 {
            anyhow::bail!("<file> required");
        }
    }

    let filename = if count.is_some() { &args[2] } else { &args[1] }.as_str();

    let mut count = count.unwrap_or(DEFAULT_COUNT);

    if let Ok(fd) = open(filename, OFlag::O_RDONLY, Mode::S_IRUSR) {
        let file_size = lseek(&fd, 0, Whence::SeekEnd)?;
        let mut pos = file_size;
        let mut res: Vec<String> = vec![];
        let mut partial_line = String::new();

        while count > 0 && pos > 0 {
            let to_read = std::cmp::min(pos, 100);
            pos -= to_read;

            let mut buf = vec![0u8; to_read as usize];
            lseek(&fd, pos, Whence::SeekSet)?;
            read(&fd, &mut buf)?;

            for &byte in buf.iter().rev() {
                if byte == b'\n' {
                    if !partial_line.is_empty() {
                        let complete: String = partial_line.chars().rev().collect();
                        res.push(complete);
                        partial_line.clear();
                        count -= 1;
                        if count == 0 {
                            break;
                        }
                    }

                    partial_line.push('\n');
                } else {
                    partial_line.push(byte as char);
                }
            }
        }

        if count > 0 && !partial_line.is_empty() {
            res.push(partial_line.chars().rev().collect());
        }

        res.reverse();
        println!("{}", res.join(""));

        close(fd)?;
    }

    Ok(())
}
