use std::ffi::{CStr, OsStr, c_char};
use std::fmt::Write;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::ptr::null_mut;

use libc::{ERANGE, gethostname, getpwuid_r, getuid, passwd};


const COLOR_STARTS: [&'static str; 7] = [
    "%F{red}",
    "%F{green}",
    "%F{yellow}",
    "%F{blue}",
    "%F{magenta}",
    "%F{cyan}",
    "%F{white}",
];


// hash the string to choose a color
fn hashify(s: &str) -> u8 {
    let mut hash = 0;
    for b in s.bytes() {
        hash ^= b;
    }
    hash
}

fn color_start_for(s: &str) -> &'static str {
    let index_u8 = hashify(s);
    let index: usize = index_u8.into();
    &COLOR_STARTS[index % COLOR_STARTS.len()]
}


fn get_hostname() -> Result<String, io::Error> {
    let mut buf = [0u8; 256];
    let ret = unsafe {
        gethostname(buf.as_mut_ptr() as *mut c_char, buf.len())
    };
    if ret == -1 {
        return Err(io::Error::last_os_error());
    }

    let hn_cstr = CStr::from_bytes_until_nul(&buf).unwrap();
    Ok(hn_cstr.to_string_lossy().into_owned())
}


fn get_username_and_home_dir(uid: u32) -> Result<(String, Option<PathBuf>), io::Error> {
    let mut string_buf_size = 1024;
    while string_buf_size < 4*1024*1024 {
        let mut string_buf = vec![0u8; string_buf_size];
        let mut pwd_structure: passwd = unsafe { std::mem::zeroed() };
        let mut result_ptr = null_mut();
        let err_num = unsafe {
            getpwuid_r(
                uid,
                &raw mut pwd_structure,
                string_buf.as_mut_ptr() as *mut i8,
                string_buf.len(),
                &raw mut result_ptr,
            )
        };
        if err_num == 0 {
            if result_ptr.is_null() {
                // user entry not found
                return Err(io::ErrorKind::NotFound.into());
            } else {
                // success
                let username_cstr = unsafe { CStr::from_ptr(pwd_structure.pw_name) };
                let username = username_cstr.to_string_lossy().into_owned();
                let home_dir = if pwd_structure.pw_dir.is_null() {
                    None
                } else {
                    let home_dir_cstr = unsafe { CStr::from_ptr(pwd_structure.pw_dir) };
                    let home_dir_os = OsStr::from_bytes(home_dir_cstr.to_bytes());
                    let home_dir = PathBuf::from(home_dir_os);
                    Some(home_dir)
                };
                return Ok((username, home_dir));
            }
        } else if err_num == ERANGE {
            // buffer too small; try again
            string_buf_size *= 2;
            continue;
        }
    }

    // we can't grow the buffer to infinity and beyond
    return Err(io::ErrorKind::FileTooLarge.into());
}


fn start_git_command() -> Command {
    let mut cmd = Command::new("git");
    cmd.env("GIT_OPTIONAL_LOCKS", "0");
    cmd
}


fn is_git_repo() -> bool {
    let git_exit_status = start_git_command()
        .arg("rev-parse")
        .arg("--git-dir")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status().unwrap();
    git_exit_status.success()
}

fn git_collect_info(args: &[&str]) -> Option<String> {
    let mut output = start_git_command();
    for arg in args {
        output.arg(*arg);
    }
    let result = output
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output().unwrap();
    if !result.status.success() {
        None
    } else {
        Some(
            String::from_utf8_lossy(result.stdout.trim_ascii())
                .into_owned()
        )
    }
}

fn ignore_git(what: &str) -> bool {
    let ignore_value = git_collect_info(&[
        "config",
        "get",
        "--type=bool",
        "--default=false",
        what,
    ]);
    match ignore_value.as_deref() {
        None => {
            // assume user is interested
            false
        },
        Some("true") => true,
        Some("false") => false,
        Some(_) => {
            // assume user is interested
            false
        },
    }
}

fn git_branch_name() -> Option<String> {
    git_collect_info(&[
        "symbolic-ref",
        "--short",
        "HEAD",
    ])
}

fn git_tag_name() -> Option<String> {
    git_collect_info(&[
        "describe",
        "--tags",
        "--exact-match",
        "HEAD",
    ])
}

fn git_whatever_name() -> Option<String> {
    git_collect_info(&[
        "rev-parse",
        "--short",
        "HEAD",
    ])
}

fn is_git_dirty() -> Option<bool> {
    let dirty_output = git_collect_info(&[
        "status",
        "--porcelain",
    ])?;
    Some(dirty_output.trim_ascii().len() > 0)
}


fn main() {
    const OB: &'static str = "{";
    const CB: &'static str = "}";

    // gimme UID and username
    let uid = unsafe { getuid() };
    let (username, _home_dir) = get_username_and_home_dir(uid).unwrap();

    // gimme hostname
    let hostname = get_hostname().unwrap();

    // git?
    let git_info = if is_git_repo() && !ignore_git("ravuprompt.ignorerepo") {
        let mut inner_git_info = if let Some(bn) = git_branch_name() {
            let bn_percent = bn.replace("%", "%%");
            format!("%F{OB}yellow{CB}%B{bn_percent}%b%f")
        } else if let Some(tn) = git_tag_name() {
            let tn_percent = tn.replace("%", "%%");
            format!("%F{OB}yellow{CB}%B{tn_percent}%b%f")
        } else if let Some(wn) = git_whatever_name() {
            let wn_percent = wn.replace("%", "%%");
            format!("%F{OB}yellow{CB}%B{wn_percent}%b%f")
        } else {
            String::new()
        };

        if !ignore_git("ravuprompt.ignoredirty") {
            if is_git_dirty().unwrap_or(false) {
                write!(inner_git_info, "%F{OB}red{CB}%B*%b%f").unwrap();
            }
        }
        inner_git_info.push(' ');
        inner_git_info
    } else {
        String::new()
    };

    // spit it out
    let username_color_start = if uid == 0 {
        // always red
        &COLOR_STARTS[0]
    } else {
        color_start_for(&username)
    };
    let hostname_color_start = color_start_for(&hostname);
    println!();
    println!("{username_color_start}%B{username}%b%f@{hostname_color_start}%B{hostname}%b%f %~");

    let dollar_sign = if uid == 0 { "#" } else { "$" };
    println!("{git_info}{dollar_sign} ");
}
