//! Running one platform program, with `posix_spawn` through `libc` rather than `std::process`:
//! the standard library's `Command` carries an environment map and builder machinery worth
//! about 25 KB on the device (RFC 010), for the one program the daemon ever runs, the
//! downloader. Two shapes: [`run`] reads the program's standard output into memory up to a
//! limit; [`run_watching`] lets the program write a file and watches that file's size, killing
//! the program when it passes the limit (an image is downloaded to RAM, chapter 11 §3).

use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::FromRawFd;
use std::path::Path;
use std::ptr;
use std::time::Duration;

/// What the program did: whether it exited with status 0, and its standard output up to
/// `max_bytes` (one byte more signals that there was more). Standard error goes where the
/// daemon's does, so procd's log keeps the program's own words.
pub struct Output {
    pub success: bool,
    pub stdout: Vec<u8>,
}

/// How a watched program ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Watched {
    /// It exited; whether with status 0.
    Exited(bool),
    /// The file grew past the limit and the program was killed.
    TooLarge,
}

/// How often the watched file is measured.
const WATCH_INTERVAL: Duration = Duration::from_millis(100);

/// Starts `program` (an absolute path) with `args`, standard input from `/dev/null`, standard
/// output to `stdout_fd` (closed in the parent afterwards when `close_after`), and a fixed,
/// minimal environment. Returns the child's pid.
fn spawn(program: &str, args: &[&str], stdout_fd: libc::c_int) -> io::Result<libc::pid_t> {
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
    let mut pid: libc::pid_t = 0;
    // SAFETY: the file-actions object is initialised, used, and destroyed here; every pointer
    // handed to posix_spawn stays alive until it returns.
    let rc = unsafe {
        let mut actions: libc::posix_spawn_file_actions_t = std::mem::zeroed();
        libc::posix_spawn_file_actions_init(&mut actions);
        libc::posix_spawn_file_actions_addopen(
            &mut actions,
            0,
            c"/dev/null".as_ptr(),
            libc::O_RDONLY,
            0,
        );
        libc::posix_spawn_file_actions_adddup2(&mut actions, stdout_fd, 1);
        let rc = libc::posix_spawn(
            &mut pid,
            program_c.as_ptr(),
            &actions,
            ptr::null(),
            argv.as_ptr(),
            envp.as_ptr(),
        );
        libc::posix_spawn_file_actions_destroy(&mut actions);
        rc
    };
    if rc != 0 {
        return Err(io::Error::from_raw_os_error(rc));
    }
    Ok(pid)
}

/// Waits for the child and says whether it exited with status 0.
fn reap(pid: libc::pid_t) -> io::Result<bool> {
    let mut status: libc::c_int = 0;
    // SAFETY: waiting for the child we spawned.
    let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
    if waited != pid {
        return Err(io::Error::last_os_error());
    }
    Ok(libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0)
}

fn kill(pid: libc::pid_t) {
    // SAFETY: our child; it is reaped by the caller.
    unsafe {
        libc::kill(pid, libc::SIGKILL);
    }
}

/// Runs `program` and reads its standard output, at most `max_bytes` plus one.
pub fn run(program: &str, args: &[&str], max_bytes: usize) -> io::Result<Output> {
    let mut fds = [0 as libc::c_int; 2];
    // SAFETY: a plain pipe; both ends are owned below.
    if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let (read_end, write_end) = (fds[0], fds[1]);
    let spawned = spawn(program, args, write_end);
    // SAFETY: both ends are ours; the write end is the child's now, the read end `File`'s.
    unsafe {
        libc::close(write_end);
    }
    let mut reader = unsafe { File::from_raw_fd(read_end) };
    let pid = spawned?;
    let mut stdout = Vec::new();
    let read = (&mut reader)
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut stdout);
    if stdout.len() > max_bytes {
        kill(pid);
    }
    drop(reader);
    let success = reap(pid)?;
    read?;
    Ok(Output { success, stdout })
}

/// Runs `program`, whose standard output goes nowhere, while watching `file`: when it grows past
/// `max_bytes` the program is killed and the answer is `TooLarge`. The file is the program's to
/// create (the downloader's `-O`).
pub fn run_watching(
    program: &str,
    args: &[&str],
    file: &Path,
    max_bytes: u64,
) -> io::Result<Watched> {
    let null = File::options().write(true).open("/dev/null")?;
    let pid = spawn(program, args, std::os::fd::AsRawFd::as_raw_fd(&null))?;
    drop(null);
    loop {
        let mut status: libc::c_int = 0;
        // SAFETY: polling the child we spawned.
        let waited = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        if waited == pid {
            return Ok(Watched::Exited(
                libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0,
            ));
        }
        if waited < 0 {
            return Err(io::Error::last_os_error());
        }
        if std::fs::metadata(file).is_ok_and(|m| m.len() > max_bytes) {
            kill(pid);
            reap(pid)?;
            return Ok(Watched::TooLarge);
        }
        std::thread::sleep(WATCH_INTERVAL);
    }
}

#[cfg(test)]
mod tests;
