extern crate nix;
extern crate bincode;
extern crate rustc_serialize;

use std::os::unix::io::RawFd;
use self::nix::unistd::ForkResult;
use self::nix::sys::socket;
use std::process::exit;
use std::error;
use std::fmt;

use bincode::SizeLimit;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Sys(nix::Errno),
    ChannelClosed,
}

impl From<nix::Error> for Error {
    fn from(t: nix::Error) -> Error {
        match t {
            nix::Error::Sys(e) => Error::Sys(e),
            _ => Error::Sys(nix::Errno::UnknownErrno),
        }
    }

}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::ChannelClosed => "Closed",
            &Error::Sys(ref errno) => errno.desc(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::ChannelClosed => write!(f, "Closed"),
            &Error::Sys(errno) => write!(f, "{:?}: {}", errno, errno.desc()),
        }
    }
}

pub struct Chan {
    start: (RawFd,RawFd),
    data:  (RawFd,RawFd),
    end:   (RawFd,RawFd),
}

impl Chan {

    pub fn set_timeout(&self, dur: Option<std::time::Duration>) -> Result<(), Error> {
        let tv = nix::sys::time::TimeVal {
            tv_sec:   match dur { Some(e) => e.as_secs() as i64, None => 0 },
            tv_usec:  match dur { Some(e) => {
                if e.subsec_nanos() < 1000 {
                    1 as i64
                } else {
                    (e.subsec_nanos() / 1000) as i64
                }
            }, None => 0},
        };

        socket::setsockopt(self.start.0, socket::sockopt::ReceiveTimeout, &tv)?;
        socket::setsockopt(self.start.1, socket::sockopt::ReceiveTimeout, &tv)?;
        socket::setsockopt(self.data.0,  socket::sockopt::ReceiveTimeout, &tv)?;
        socket::setsockopt(self.data.1,  socket::sockopt::ReceiveTimeout, &tv)?;
        socket::setsockopt(self.end.0,   socket::sockopt::ReceiveTimeout, &tv)?;
        socket::setsockopt(self.end.1,   socket::sockopt::ReceiveTimeout, &tv)?;

        return Ok(())
    }

    pub fn send<T>(&self, t:&T) -> Result<(), Error>  where T: rustc_serialize::Encodable {
        let encoded: Vec<u8> = bincode::rustc_serialize::encode(t, SizeLimit::Bounded(8000)).unwrap();
        self._write(&encoded[..])
    }

    fn _write(&self, b: &[u8]) -> Result<(), Error>{
        let mut void = [0;1];

        let u = socket::recv(self.start.0, &mut void, socket::MSG_EOR)?;
        if  u < 1 {
            return Err(Error::ChannelClosed)
        }
        socket::send(self.data.1, b, socket::MSG_EOR)?;
        let u = socket::recv(self.end.0, &mut void,    socket::MSG_EOR)?;
        if  u < 1 {
            return Err(Error::ChannelClosed)
        }

        return Ok(());
    }

    pub fn recv<T>(&self) -> Result<T, Error> where T: rustc_serialize::Decodable {
        let mut buf = [0;8000];

        socket::send(self.start.1, &[0;1]  , socket::MSG_EOR )?;
        let u = socket::recv(self.data.0, &mut buf, socket::MSG_EOR )?;
        if  u < 1 {
            return Err(Error::ChannelClosed)
        }
        socket::send(self.end.1, &[0;1]  , socket::MSG_EOR )?;

        let decoded: T = bincode::rustc_serialize::decode(&buf[..]).unwrap();
        Ok(decoded)
    }

    pub fn close(&self) -> Result<(), Error> {
        socket::send(self.start.1, &[0;0], socket::MSG_EOR)?;
        socket::send(self.data.1,  &[0;0], socket::MSG_EOR)?;
        socket::send(self.end.1,   &[0;0], socket::MSG_EOR)?;
        return Ok(())
    }
}

pub fn channel() -> Result<Chan, nix::Error> {
    let start = socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty());
    let data  = socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty());
    let end   = socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty());
    Ok(Chan {start: start?, data: data?, end: end? })
}

pub struct Proc {
}


pub fn fork<F>(start: F) -> Proc where F: Fn(){
    if let ForkResult::Child = nix::unistd::fork().expect("fork failed") {
        start();
        exit_must_not_continue();
    };
    Proc{}
}

#[allow(unreachable_code)]
fn exit_must_not_continue() ->!{
    exit(0);
    loop{}
}

