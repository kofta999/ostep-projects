use nix::{
    sys::stat::{FileStat, stat},
    unistd::{Gid, Group, Uid, User},
};
mod util;
use util::{format_permissions, get_file_type};

use crate::util::format_time;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Err("Filename needed".into());
    }

    let filename = &args[1];

    let FileStat {
        st_dev,
        st_ino,
        st_nlink,
        st_mode,
        st_uid,
        st_gid,
        st_rdev,
        st_size,
        st_blocks,
        st_blksize,
        st_atime,
        st_atime_nsec,
        st_mtime,
        st_mtime_nsec,
        st_ctime,
        st_ctime_nsec,
        ..
    } = stat(filename.as_str()).map_err(|e| e.to_string())?;

    let mode = st_mode & 0o7777;
    let username = User::from_uid(Uid::from_raw(st_uid)).unwrap().unwrap().name;
    let group_name = Group::from_gid(Gid::from_raw(st_gid))
        .unwrap()
        .unwrap()
        .name;

    let stats_string = format!(
        "
  File: {filename}
  Size: {st_size}           Blocks: {st_blocks}         IO Block: {st_blksize}  {}
Device: {st_rdev},{st_dev}      Inode:  {st_ino}      Links: {st_nlink}
Access: ({mode:04o}/{})   Uid: ({st_uid}/   {username})    Gid: ({st_gid}/   {group_name})
Access: {}
Modify: {}
Change: {}
       ",
        get_file_type(st_mode),
        format_permissions(st_mode),
        format_time(st_atime, st_atime_nsec),
        format_time(st_mtime, st_mtime_nsec),
        format_time(st_ctime, st_ctime_nsec),
    );

    println!("{stats_string}");

    Ok(())
}
