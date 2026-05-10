use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::process;

const SCM_RIGHTS: i32 = 1;

fn recv_fd(sock: RawFd) -> Result<RawFd, String> {
    let mut data: u8 = 0;
    let mut iov = libc::iovec {
        iov_base: &mut data as *mut u8 as *mut libc::c_void,
        iov_len: 1,
    };

    let cmsg_len = unsafe { libc::CMSG_SPACE(std::mem::size_of::<RawFd>() as u32) };
    let mut cmsg_buf: Vec<u8> = vec![0u8; cmsg_len as usize];

    let mut msgh = libc::msghdr {
        msg_name: std::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut iov as *mut libc::iovec,
        msg_iovlen: 1,
        msg_control: cmsg_buf.as_mut_ptr() as *mut libc::c_void,
        msg_controllen: cmsg_len as usize,
        msg_flags: 0,
    };

    let ret = unsafe { libc::recvmsg(sock, &mut msgh, 0) };
    if ret < 0 {
        return Err(format!("recvmsg failed: {}", std::io::Error::last_os_error()));
    }

    let cmsg = unsafe { libc::CMSG_FIRSTHDR(&msgh) };
    if cmsg.is_null()
        || unsafe { (*cmsg).cmsg_level } != libc::SOL_SOCKET
        || unsafe { (*cmsg).cmsg_type } != SCM_RIGHTS
    {
        return Err("invalid cmsg received".to_string());
    }

    let mut fd: RawFd = -1;
    unsafe {
        std::ptr::copy(libc::CMSG_DATA(cmsg) as *const RawFd, &mut fd, 1);
    }
    Ok(fd)
}

fn relay_io(pty_fd: RawFd) {
    let pty_fd_read = pty_fd;
    let pty_fd_write = pty_fd;

    let pty_fd_clone = unsafe { libc::dup(pty_fd) };
    if pty_fd_clone < 0 {
        eprintln!("sgsh-connect: dup failed");
        process::exit(1);
    }

    let relay_thread = std::thread::spawn(move || {
        let mut pty = unsafe { std::fs::File::from_raw_fd(pty_fd_read) };
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        let mut buf = [0u8; 4096];
        loop {
            match pty.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if lock.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    let _ = lock.flush();
                }
                Err(_) => break,
            }
        }
    });

    let mut pty_out = unsafe { std::fs::File::from_raw_fd(pty_fd_clone) };
    let stdin = std::io::stdin();
    let mut stdin_handle = stdin.lock();
    let mut buf = [0u8; 4096];
    loop {
        match stdin_handle.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if pty_out.write_all(&buf[..n]).is_err() {
                    break;
                }
                let _ = pty_out.flush();
            }
            Err(_) => break,
        }
    }

    let _ = relay_thread.join();

    unsafe { libc::close(pty_fd_write); }
}

fn main() {
    let socket_path = std::env::var("GATE_SOCKET")
        .unwrap_or_else(|_| "/run/gate.sock".to_string());

    let mut stream = match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("sgsh-connect: cannot connect to {}: {}", socket_path, e);
            process::exit(1);
        }
    };

    let request = serde_json::json!({
        "type": "spawn_shell"
    });

    if let Err(e) = serde_json::to_writer(&stream, &request) {
        eprintln!("sgsh-connect: failed to send request: {}", e);
        process::exit(1);
    }
    if let Err(e) = stream.write_all(b"\n") {
        eprintln!("sgsh-connect: failed to send newline: {}", e);
        process::exit(1);
    }

    let raw_fd = stream.as_raw_fd();
    let pty_fd = match recv_fd(raw_fd) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("sgsh-connect: failed to receive pty fd: {}", e);
            process::exit(1);
        }
    };

    relay_io(pty_fd);

    unsafe { libc::close(pty_fd); }
}
