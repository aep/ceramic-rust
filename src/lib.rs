extern crate nix;
extern crate bincode;
extern crate rustc_serialize;

use std::os::unix::io::RawFd;
use self::nix::unistd::ForkResult;
use self::nix::sys::socket;
use std::process::exit;
use self::nix::unistd;

use bincode::SizeLimit;

pub struct Chan {
    start: (RawFd,RawFd),
    data:  (RawFd,RawFd),
    end:   (RawFd,RawFd),
}

impl Chan {
    pub fn send<T>(&self, t:&T) -> bool where T: rustc_serialize::Encodable {
        let encoded: Vec<u8> = bincode::rustc_serialize::encode(t, SizeLimit::Bounded(8000)).unwrap();

        let mut void = [0;1];

        //println!("{} r- start.0", unistd::getpid());
        let u = socket::recv(self.start.0, &mut void,    socket::MSG_EOR).expect("recv failed");
        //println!("{} r+ start.0", unistd::getpid());
        if u != 1 {
            return false
        };

        //println!("{} w- data.0", unistd::getpid());
        let u = socket::send(self.data.0, &encoded[..], socket::MSG_EOR).expect("send failed");
        //println!("{} w+ data.0", unistd::getpid());
        if u < 1 {
            return false;
        };

        //println!("{} r-  end.0", unistd::getpid());
        let u = socket::recv(self.end.0, &mut void,    socket::MSG_EOR).expect("recv failed");
        //println!("{} r+  end.0", unistd::getpid());
        if u != 1 {
            return false;
        };

        return true;
    }

    pub fn recv<T>(&self) -> Result<T, nix::Error> where T: rustc_serialize::Decodable {

        let mut buf = [0;8000];
        //println!("{} w- start.1", unistd::getpid());
        let u = socket::send(self.start.1, &[0;1]  , socket::MSG_EOR ).expect("send failed");
        //println!("{} w+ start.1", unistd::getpid());
        if u != 1 {
            return Err(nix::Error::Sys(nix::Errno::UnknownErrno))
        }

        //println!("{} r- data.1", unistd::getpid());
        let u = socket::recv(self.data.1, &mut buf, socket::MSG_EOR ).expect("recv failed");
        //println!("{} r+ data.1", unistd::getpid());
        if u < 1  {
            return Err(nix::Error::Sys(nix::Errno::UnknownErrno))
        }

        //println!("{} w-  end.1", unistd::getpid());
        let u = socket::send(self.end.1, &[0;1]  , socket::MSG_EOR ).expect("send failed");
        //println!("{} w+  end.1", unistd::getpid());
        if u != 1 {
            return Err(nix::Error::Sys(nix::Errno::UnknownErrno))
        }

        let decoded: T = bincode::rustc_serialize::decode(&buf[..]).unwrap();
        Ok(decoded)
    }

    pub fn new() -> Chan {
        Chan {
            start: socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty()).expect("socketpair failed"),
            data:  socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty()).expect("socketpair failed"),
            end:   socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty()).expect("socketpair failed"),
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


