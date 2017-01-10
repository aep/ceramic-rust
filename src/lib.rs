extern crate nix;
extern crate bincode;
extern crate rustc_serialize;

use std::os::unix::io::RawFd;
use self::nix::unistd::ForkResult;
use self::nix::sys::socket;
use std::process::exit;

use bincode::SizeLimit;

pub struct Chan {
    sync: (RawFd,RawFd),
    data: (RawFd,RawFd),
}

impl Chan {
    pub fn send<T>(&self, t:&T) -> bool where T: rustc_serialize::Encodable {
        let encoded: Vec<u8> = bincode::rustc_serialize::encode(t, SizeLimit::Bounded(8000)).unwrap();

        let mut void = [0;1];
        let u = socket::recv(self.sync.0, &mut void,    socket::MSG_EOR).expect("recv failed");
        if u != 1 {
            return false
        }
        let u = socket::send(self.data.0, &encoded[..], socket::MSG_EOR).expect("send failed");
        return u > 0
    }

    pub fn recv<T>(&self) -> Result<T, nix::Error> where T: rustc_serialize::Decodable {

        let mut buf = [0;8000];
        let u = socket::send(self.sync.1, &[0;1]  , socket::MSG_EOR ).expect("send failed");
        if u != 1 {
            return Err(nix::Error::Sys(nix::Errno::UnknownErrno))
        }
        let u = socket::recv(self.data.1, &mut buf, socket::MSG_EOR ).expect("recv failed");
        if u < 1  {
            return Err(nix::Error::Sys(nix::Errno::UnknownErrno))
        }
        let decoded: T = bincode::rustc_serialize::decode(&buf[..]).unwrap();
        Ok(decoded)
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

