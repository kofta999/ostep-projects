use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{Whence, close, lseek, read};

const DEFAULT_COUNT: i32 = 10;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err("Missing arguments".into());
    }
    let mut count: Option<i32> = None;

    if args[1].starts_with("-") {
        let temp = args[1][1..]
            .parse::<i32>()
            .map_err(|_| "Count should be a number")?;

        count = Some(temp);

        if args.len() != 3 {
            return Err("Filename required".into());
        }
    }

    let filename = if count.is_some() { &args[2] } else { &args[1] }.as_str();

    let mut count = count.unwrap_or(DEFAULT_COUNT);

    if let Ok(fd) = open(filename, OFlag::O_RDONLY, Mode::S_IRUSR) {
        let file_size = lseek(&fd, 0, Whence::SeekEnd)
            .map_err(|_| "Error while seeking to the end of the file")?;
        let mut pos = file_size;
        let mut res: Vec<String> = vec![];
        let mut partial_line = String::new();

        while count > 0 && pos > 0 {
            let to_read = std::cmp::min(pos, 100);
            pos -= to_read;

            let mut buf = vec![0u8; to_read as usize];
            lseek(&fd, pos, Whence::SeekSet).map_err(|_| "Could not seek to position")?;
            read(&fd, &mut buf).map_err(|_| "Could not read to the buffer")?;

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

        close(fd).map_err(|_| "Could not close the file")?;
    }

    Ok(())
}
