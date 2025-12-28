use std::os::fd::{AsRawFd, IntoRawFd, OwnedFd};

use nix::errno::Errno;

pub type TTY = OwnedFd;

pub fn open(vt: u16) -> Result<TTY, Errno> {
    let fd = nix::fcntl::open(
        format!("/dev/tty{}", vt).as_str(),
        nix::fcntl::OFlag::O_RDWR | nix::fcntl::OFlag::O_NOCTTY,
        nix::sys::stat::Mode::from_bits_truncate(0o666),
    )?;

    Ok(fd)
}

pub fn close(tty: TTY) {
    unsafe {
        libc::close(tty.into_raw_fd());
    }
}

pub fn current(fd: &TTY) -> u16 {
    #[allow(dead_code, non_camel_case_types)]
    #[repr(C)]
    struct vt_state {
        pub v_active: u16,
        pub v_signal: u16,
        pub v_state: u16,
    }
    let mut state = vt_state {
        v_active: 0,
        v_signal: 0,
        v_state: 0,
    };

    unsafe {
        nix::libc::ioctl(
            fd.as_raw_fd(),
            0x5603 as nix::sys::ioctl::ioctl_num_type,
            &mut state,
        );
    }

    state.v_active
}

pub fn switch(fd: &TTY, vt: u16) {
    #[allow(dead_code, non_camel_case_types)]
    #[repr(C)]
    struct vt_mode {
        pub mode: u8,
        pub waitv: u8,
        pub relsig: u16,
        pub acqsig: u16,
        pub frsig: u16,
    }

    #[allow(dead_code, non_camel_case_types)]
    #[repr(C)]
    struct vt_setactivate {
        pub console: u64,
        pub mode: vt_mode,
    }

    let setactivate = vt_setactivate {
        console: vt as u64,
        mode: vt_mode {
            mode: 0,
            waitv: 0,
            relsig: 0,
            acqsig: 0,
            frsig: 0,
        },
    };

    unsafe {
        nix::libc::ioctl(
            fd.as_raw_fd(),
            0x560F as nix::sys::ioctl::ioctl_num_type,
            &setactivate,
        );
    }

    unsafe {
        nix::libc::ioctl(
            fd.as_raw_fd(),
            0x5607 as nix::sys::ioctl::ioctl_num_type,
            vt as u64,
        );
    }
}

pub fn take(fd: &TTY) {
    unsafe {
        nix::libc::ioctl(fd.as_raw_fd(), 0x540E as nix::sys::ioctl::ioctl_num_type, 1);
    }
}
