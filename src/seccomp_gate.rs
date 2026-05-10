use std::ffi::CString;
use std::io;
use std::os::unix::io::RawFd;

use libc::{c_int, c_void, pid_t, CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_SPACE};
use libseccomp::{ScmpAction, ScmpFilterContext, ScmpNotifReq, ScmpNotifResp, ScmpNotifRespFlags, ScmpSyscall};

use crate::pipeline::PipelineResult;
use crate::types::{GateError, SeccompNotification};

const SCM_RIGHTS: c_int = 1;

#[allow(clippy::missing_safety_doc)]
pub unsafe fn fork_and_trap(
    shell_binary: &str,
    pty_master: RawFd,
    pty_slave: RawFd,
    sock_fd: RawFd,
) -> Result<pid_t, GateError> {
    let pid = libc::fork();
    if pid < 0 {
        return Err("fork failed".into());
    }

    if pid == 0 {
        child_trap_and_exec(shell_binary, pty_master, pty_slave, sock_fd);
    }

    Ok(pid)
}

unsafe fn child_trap_and_exec(
    shell_binary: &str,
    pty_master: RawFd,
    pty_slave: RawFd,
    sock_fd: RawFd,
) -> ! {
    libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);

    let mut filter = match ScmpFilterContext::new(ScmpAction::Allow) {
        Ok(f) => f,
        Err(_) => libc::_exit(1),
    };

    if filter.add_rule(
        ScmpAction::Notify,
        ScmpSyscall::from_name("execve").unwrap_or_else(|_| libc::_exit(1)),
    ).is_err() {
        libc::_exit(1);
    }

    if filter.add_rule(
        ScmpAction::Notify,
        ScmpSyscall::from_name("execveat").unwrap_or_else(|_| libc::_exit(1)),
    ).is_err() {
        libc::_exit(1);
    }

    if filter.add_rule(
        ScmpAction::Errno(libc::EPERM),
        ScmpSyscall::from_name("seccomp").unwrap_or_else(|_| libc::_exit(1)),
    ).is_err() {
        libc::_exit(1);
    }

    if filter.load().is_err() {
        libc::_exit(1);
    }

    let notify_fd = match filter.get_notify_fd() {
        Ok(fd) => fd,
        Err(_) => libc::_exit(1),
    };

    let _ = libc::close(pty_master);

    libc::setsid();

    if libc::ioctl(pty_slave, libc::TIOCSCTTY, 0) < 0 {
        libc::_exit(1);
    }

    libc::dup2(pty_slave, 0);
    libc::dup2(pty_slave, 1);
    libc::dup2(pty_slave, 2);
    if pty_slave > 2 {
        let _ = libc::close(pty_slave);
    }

    libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0);

    let data: u8 = 0;
    let iov = libc::iovec {
        iov_base: &data as *const u8 as *mut c_void,
        iov_len: 1,
    };

    let cmsg_len = CMSG_SPACE(std::mem::size_of::<RawFd>() as u32);
    let mut cmsg_buf: Vec<u8> = vec![0u8; cmsg_len as usize];

    let msgh = libc::msghdr {
        msg_name: std::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &iov as *const libc::iovec as *mut libc::iovec,
        msg_iovlen: 1,
        msg_control: cmsg_buf.as_mut_ptr() as *mut c_void,
        msg_controllen: cmsg_len as usize,
        msg_flags: 0,
    };

    let cmsg = CMSG_FIRSTHDR(&msgh);
    (*cmsg).cmsg_level = libc::SOL_SOCKET;
    (*cmsg).cmsg_type = SCM_RIGHTS;
    (*cmsg).cmsg_len = CMSG_LEN(std::mem::size_of::<RawFd>() as u32) as usize;
    std::ptr::copy(&notify_fd, CMSG_DATA(cmsg) as *mut RawFd, 1);

    let _ = libc::sendmsg(sock_fd, &msgh, 0);
    let _ = libc::close(sock_fd);
    let _ = libc::close(notify_fd);

    let shell_cstr = CString::new(shell_binary).unwrap_or_else(|_| libc::_exit(1));
    let shell_argv = [shell_cstr.as_ptr(), std::ptr::null()];
    let envp: [*const libc::c_char; 1] = [std::ptr::null()];

    libc::execve(shell_cstr.as_ptr(), shell_argv.as_ptr(), envp.as_ptr());
    libc::_exit(127);
}

pub fn receive_notification(fd: RawFd) -> Result<SeccompNotification, GateError> {
    let req = ScmpNotifReq::receive(fd)?;
    Ok(SeccompNotification {
        id: req.id,
        pid: req.pid,
        syscall_nr: req.data.syscall.into(),
        args: req.data.args,
        instruction_pointer: req.data.instr_pointer,
    })
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn read_target_argv(pid: u32, argv_ptr: u64) -> Result<Vec<String>, GateError> {
    let mem_path = format!("/proc/{}/mem", pid);
    let fd = libc::open(
        CString::new(mem_path)?.as_ptr(),
        libc::O_RDONLY,
    );
    if fd < 0 {
        return Err(format!(
            "failed to open /proc/{}/mem: {}",
            pid,
            io::Error::last_os_error()
        )
        .into());
    }

    let result = read_argv_from_mem(fd, argv_ptr);
    let _ = libc::close(fd);
    result
}

unsafe fn read_argv_from_mem(mem_fd: RawFd, argv_ptr: u64) -> Result<Vec<String>, GateError> {
    let mut argv = Vec::new();
    let mut offset = argv_ptr;

    loop {
        let ptr = read_u64_from_mem(mem_fd, offset)?;
        if ptr == 0 {
            break;
        }
        let arg = read_cstring_from_mem(mem_fd, ptr)?;
        argv.push(arg);
        offset += 8;
    }

    Ok(argv)
}

unsafe fn read_u64_from_mem(mem_fd: RawFd, offset: u64) -> Result<u64, GateError> {
    let mut buf = [0u8; 8];
    let n = libc::pread(mem_fd, buf.as_mut_ptr() as *mut c_void, 8, offset as i64);
    if n < 8 {
        return Err(format!("short read at 0x{:x}: {} bytes", offset, n).into());
    }
    Ok(u64::from_le_bytes(buf))
}

unsafe fn read_cstring_from_mem(mem_fd: RawFd, addr: u64) -> Result<String, GateError> {
    let mut buf = vec![0u8; 4096];
    let n = libc::pread(mem_fd, buf.as_mut_ptr() as *mut c_void, 4096, addr as i64);
    if n <= 0 {
        return Err(format!("failed to read string at 0x{:x}", addr).into());
    }
    let len = buf[..n as usize]
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(n as usize);
    Ok(String::from_utf8_lossy(&buf[..len]).to_string())
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn create_pty() -> Result<(RawFd, RawFd), GateError> {
    let mut master: RawFd = -1;
    let mut slave: RawFd = -1;

    let ret = libc::openpty(
        &mut master as *mut RawFd,
        &mut slave as *mut RawFd,
        std::ptr::null_mut(),
        std::ptr::null(),
        std::ptr::null(),
    );
    if ret < 0 {
        return Err(format!("openpty failed: {}", io::Error::last_os_error()).into());
    }

    Ok((master, slave))
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn create_socketpair() -> Result<(RawFd, RawFd), GateError> {
    let mut fds = [0i32; 2];
    let ret = libc::socketpair(libc::AF_UNIX, libc::SOCK_SEQPACKET, 0, fds.as_mut_ptr());
    if ret < 0 {
        return Err(format!(
            "socketpair failed: {}",
            io::Error::last_os_error()
        )
        .into());
    }
    Ok((fds[0], fds[1]))
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn recv_fd(sock: RawFd) -> Result<RawFd, GateError> {
    let mut data: u8 = 0;
    let iov = libc::iovec {
        iov_base: &mut data as *mut u8 as *mut c_void,
        iov_len: 1,
    };

    let cmsg_len = CMSG_SPACE(std::mem::size_of::<RawFd>() as u32);
    let mut cmsg_buf: Vec<u8> = vec![0u8; cmsg_len as usize];

    let mut msgh = libc::msghdr {
        msg_name: std::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &iov as *const libc::iovec as *mut libc::iovec,
        msg_iovlen: 1,
        msg_control: cmsg_buf.as_mut_ptr() as *mut c_void,
        msg_controllen: cmsg_len as usize,
        msg_flags: 0,
    };

    let ret = libc::recvmsg(sock, &mut msgh, 0);
    if ret < 0 {
        return Err(format!(
            "recvmsg failed: {}",
            io::Error::last_os_error()
        )
        .into());
    }

    let cmsg = CMSG_FIRSTHDR(&msgh);
    if cmsg.is_null()
        || (*cmsg).cmsg_level != libc::SOL_SOCKET
        || (*cmsg).cmsg_type != SCM_RIGHTS
    {
        return Err("invalid cmsg received".into());
    }

    let mut fd: RawFd = -1;
    std::ptr::copy(CMSG_DATA(cmsg) as *const RawFd, &mut fd, 1);
    Ok(fd)
}

pub fn respond_continue(fd: RawFd, id: u64) -> Result<(), GateError> {
    let flags = ScmpNotifRespFlags::from_bits(0x1)
        .ok_or("failed to create CONTINUE flag")?;
    let resp = ScmpNotifResp::new_val(id, 0, flags);
    resp.respond(fd)?;
    Ok(())
}

pub fn respond_block(fd: RawFd, id: u64, errno: i32) -> Result<(), GateError> {
    let resp = ScmpNotifResp::new_error(id, -errno, ScmpNotifRespFlags::empty());
    resp.respond(fd)?;
    Ok(())
}

#[allow(dead_code)]
pub fn notify_id_valid(fd: RawFd, id: u64) -> Result<(), GateError> {
    libseccomp::notify_id_valid(fd, id)?;
    Ok(())
}

pub fn pipeline_result_to_syscall_response(result: &PipelineResult) -> (bool, Option<i32>) {
    if result.allowed {
        (true, None)
    } else {
        (false, Some(libc::EPERM))
    }
}

pub fn close_fd(fd: RawFd) {
    unsafe {
        let _ = libc::close(fd);
    }
}
