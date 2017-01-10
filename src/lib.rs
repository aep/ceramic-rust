extern crate nix;

use std::os::unix::io::RawFd;
use self::nix::unistd::ForkResult;
use self::nix::sys::socket;
use self::nix::unistd;
use std::process::exit;

pub struct Chan {
    sync: (RawFd,RawFd),
    data: (RawFd,RawFd),
}

impl Chan {
    pub fn send(&self, buf: &[u8]) {
        let mut void = [0;1];
        unistd::read  (self.sync.0, &mut void);
        unistd::write (self.data.0, buf);
    }

    pub fn recv(&self, buf: &mut [u8]) {
        unistd::write (self.sync.1, &[0;1]);
        unistd::read  (self.data.1, buf);
    }

    pub fn new() -> Chan {
        Chan {
            sync: socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty()).expect("socketpair failed"),
            data: socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty()).expect("socketpair failed"),
        }
    }

}

pub struct Proc {
}

impl Proc {
    pub fn new<F>(start: F) -> Proc where F: Fn(){
        if let ForkResult::Child = nix::unistd::fork().expect("fork failed") {
            start();
            exit(0);
        };
        Proc{}
    }
}

