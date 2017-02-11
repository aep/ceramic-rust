extern crate nix;
extern crate bincode;
extern crate rustc_serialize;

use std::os::unix::io::RawFd;
use self::nix::unistd::ForkResult;
use self::nix::sys::socket;
use self::nix::libc::pid_t;
use self::nix::sys::signal::kill;
use nix::sys::signal::Signal;
use std::process::exit;
use nix::sys::wait;

use bincode::SizeLimit;

pub struct Chan<T> where T: rustc_serialize::Decodable {
    start: (RawFd,RawFd),
    data:  (RawFd,RawFd),
    end:   (RawFd,RawFd),
    phantom: std::marker::PhantomData<T>,
}

impl<T> Iterator for Chan<T> where T: rustc_serialize::Decodable {
    type Item = Result<T, nix::Error>;

    fn next(&mut self) -> Option<Result<T, nix::Error>> {
        let r = self.recv();
        match r {
            Err(e)      => Some(Err(e)),
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None)    => None,
        }
    }
}

impl<T> Chan<T> where T: rustc_serialize::Decodable {

    pub fn set_timeout(&self, dur: Option<std::time::Duration>) -> Result<(), nix::Error> {
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

    pub fn send(&self, t:&T) -> Result<(), nix::Error>  where T: rustc_serialize::Encodable {
        let encoded: Vec<u8> = bincode::rustc_serialize::encode(t, SizeLimit::Bounded(8000)).unwrap();
        self._write(&encoded[..])
    }

    fn _write(&self, b: &[u8]) -> Result<(), nix::Error>{
        let mut void = [0;1];

        socket::recv(self.start.0, &mut void, socket::MSG_EOR)?;
        socket::send(self.data.1, b, socket::MSG_EOR)?;
        socket::recv(self.end.0, &mut void,    socket::MSG_EOR)?;

        return Ok(());
    }

    pub fn recv(&self) -> Result<Option<T>, nix::Error> where T: rustc_serialize::Decodable {
        let mut buf = [0;8000];

        socket::send(self.start.1, &[0;1]  , socket::MSG_EOR )?;
        let u = socket::recv(self.data.0, &mut buf, socket::MSG_EOR )?;
        if  u < 1 {
            return Ok(None);
        }
        socket::send(self.end.1, &[0;1]  , socket::MSG_EOR )?;

        let decoded: T = bincode::rustc_serialize::decode(&buf[..]).unwrap();
        Ok(Some(decoded))
    }

    pub fn close(&self) -> Result<(), nix::Error> {
        //TODO: this might not be the correct semantics. double check the theory behind close
        socket::send(self.start.1, &[0;0], socket::MSG_EOR)?;
        socket::send(self.data.1,  &[0;0], socket::MSG_EOR)?;
        socket::send(self.end.1,   &[0;0], socket::MSG_EOR)?;
        return Ok(())
    }
}

pub fn channel<T>() -> Result<Chan<T>, nix::Error>  where T: rustc_serialize::Decodable {
    let start = socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty());
    let data  = socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty());
    let end   = socket::socketpair(socket::AddressFamily::Unix, socket::SockType::Datagram, 0, socket::SockFlag::empty());
    Ok(Chan {start: start?, data: data?, end: end? , phantom: std::marker::PhantomData})
}

pub struct Proc {
    pid: pid_t
}

impl Drop for Proc {
    fn drop(&mut self) {
        if wait::waitpid(self.pid, Some(wait::WNOHANG)) == Ok(wait::WaitStatus::StillAlive) {
            kill(self.pid, Signal::SIGTERM).unwrap();
        }

    }
}


pub fn fork<F>(start: F) -> Proc where F: Fn(){
    match nix::unistd::fork().expect("fork failed") {
        ForkResult::Child => {
            start();
            exit_must_not_continue();
        }
        ForkResult::Parent{child} => {
            Proc{pid:child}
        }
    }
}

#[allow(unreachable_code)]
fn exit_must_not_continue() ->!{
    exit(0);
    loop{}
}

