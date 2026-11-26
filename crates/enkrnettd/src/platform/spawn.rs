//! Running one platform program and capturing its output, with `posix_spawn` through `libc`
//! rather than `std::process`: the standard library's `Command` carries an environment map and
//! builder machinery worth about 25 KB on the device (RFC 010), for the one program the daemon
//! ever runs, the downloader.

use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::FromRawFd;
use std::ptr;

/// What the program did: whether it exited with status 0, and its standard output up to
/// `max_bytes` (one byte more signals that there was more). Standard error goes where the
/// daemon's does, so procd's log keeps the program's own words.
pub struct Output {
    pub success: bool,
    pub stdout: Vec<u8>,
}

/// Runs `program` (an absolute path) with `args`, standard input from `/dev/null`, and reads its
/// standard output. The environment is a fixed, minimal one.
pub fn run(program: &str, args: &[&str], max_bytes: usize) -> io::Result<Output> {
    let program_c = CString::new(program).map_err(|_| io::Error::other("nul in program"))?;
    let mut argv_c: Vec<CString> = Vec::with_capacity(args.len() + 1);
    argv_c.push(program_c.clone());
    for a in args {
        argv_c.push(CString::new(*a).map_err(|_| io::Error::other("nul in argument"))?);
    }
    let mut argv: Vec<*mut libc::c_char> = argv_c.iter().map(|a| a.as_ptr() as *mut _).collect();
    argv.push(ptr::null_mut());
    let env_c = [c"PATH=/usr/sbin:/usr/bin:/sbin:/bin", c"HOME=/tmp"];
    let mut envp: Vec<*mut libc::c_char> = env_c.iter().map(|e| e.as_ptr() as *mut _).collect();
    envp.push(ptr::null_mut());

    let mut fds = [0 as libc::c_int; 2];
    // SAFETY: a plain pipe; both ends are owned below.
    if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let (read_end, write_end) = (fds[0], fds[1]);
    let mut pid: libc::pid_t = 0;
    // SAFETY: the file-actions object is initialised, used, and destroyed here; every pointer
    // handed to posix_spawn stays alive until it returns.
    let spawned = unsafe {
        let mut actions: libc::posix_spawn_file_actions_t = std::mem::zeroed();
        libc::posix_spawn_file_actions_init(&mut actions);
        libc::posix_spawn_file_actions_addopen(
            &mut actions,
            0,
            c"/dev/null".as_ptr(),
            libc::O_RDONLY,
            0,
        );
        libc::posix_spawn_file_actions_adddup2(&mut actions, write_end, 1);
        let rc = libc::posix_spawn(
            &mut pid,
            program_c.as_ptr(),
            &actions,
            ptr::null(),
            argv.as_ptr(),
            envp.as_ptr(),
        );
        libc::posix_spawn_file_actions_destroy(&mut actions);
        libc::close(write_end);
        rc
    };
    // SAFETY: the read end is ours alone from here; `File` closes it.
    let mut reader = unsafe { File::from_raw_fd(read_end) };
    if spawned != 0 {
        return Err(io::Error::from_raw_os_error(spawned));
    }
    let mut stdout = Vec::new();
    let read = (&mut reader)
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut stdout);
    if stdout.len() > max_bytes {
        // SAFETY: our child; it is reaped below.
        unsafe {
            libc::kill(pid, libc::SIGKILL);
        }
    }
    drop(reader);
    let mut status: libc::c_int = 0;
    // SAFETY: waiting for the child we spawned.
    let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
    read?;
    if waited != pid {
        return Err(io::Error::last_os_error());
    }
    Ok(Output {
        success: libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0,
        stdout,
    })
}

#[cfg(test)]
mod tests;
