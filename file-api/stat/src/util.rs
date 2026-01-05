use chrono::{DateTime, Local};
use nix::{
    libc::{S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFREG, S_IFSOCK},
    sys::stat::Mode,
};

pub fn get_file_type(raw_mode: u32) -> String {
    let file_type = raw_mode & S_IFMT;

    match file_type {
        S_IFREG => "regular file",
        S_IFDIR => "directory",
        S_IFLNK => "symbolic link",
        S_IFCHR => "character special file",
        S_IFBLK => "block special file",
        S_IFIFO => "fifo",
        S_IFSOCK => "socket",
        _ => "unknown",
    }
    .into()
}

pub fn format_permissions(raw_mode: u32) -> String {
    let mode = Mode::from_bits_truncate(raw_mode);
    let mut s = String::with_capacity(10);

    let type_char = match raw_mode & S_IFMT {
        S_IFREG => '-',
        S_IFDIR => 'd',
        S_IFLNK => 'l',
        S_IFBLK => 'b',
        S_IFCHR => 'c',
        S_IFIFO => 'p',
        S_IFSOCK => 's',
        _ => '?',
    };

    // Helper for special bits (SetUID/SetGID/Sticky)
    let bit_char = |has_perm: bool, has_special: bool, special_char: char| {
        match (has_perm, has_special) {
            (true, true) => special_char.to_ascii_lowercase(), // 's' or 't'
            (false, true) => special_char.to_ascii_uppercase(), // 'S' or 'T'
            (true, false) => 'x',
            (false, false) => '-',
        }
    };

    s.push(type_char);

    // Owner
    s.push(if mode.contains(Mode::S_IRUSR) {
        'r'
    } else {
        '-'
    });
    s.push(if mode.contains(Mode::S_IWUSR) {
        'w'
    } else {
        '-'
    });
    s.push(bit_char(
        mode.contains(Mode::S_IXUSR),
        mode.contains(Mode::S_ISUID),
        'S',
    ));

    // Group
    s.push(if mode.contains(Mode::S_IRGRP) {
        'r'
    } else {
        '-'
    });
    s.push(if mode.contains(Mode::S_IWGRP) {
        'w'
    } else {
        '-'
    });
    s.push(bit_char(
        mode.contains(Mode::S_IXGRP),
        mode.contains(Mode::S_ISGID),
        'S',
    ));

    // Others
    s.push(if mode.contains(Mode::S_IROTH) {
        'r'
    } else {
        '-'
    });
    s.push(if mode.contains(Mode::S_IWOTH) {
        'w'
    } else {
        '-'
    });
    s.push(bit_char(
        mode.contains(Mode::S_IXOTH),
        mode.contains(Mode::S_ISVTX),
        'T',
    ));

    s
}

pub fn format_time(time_secs: i64, time_nsecs: i64) -> String {
    let datetime = DateTime::from_timestamp(time_secs, time_nsecs.try_into().unwrap())
        .expect("Invalid Timestamp")
        .with_timezone(&Local);

    datetime.format("%m-%d-%y %T%.9f %z").to_string()
}
