#[cfg(test)]

extern crate ceramic;
extern crate rain;

mod tests {
    use ceramic;

    #[test]
    fn recv_on_main_send_in_subproc() {
        let chan   = ceramic::channel().unwrap();

        ceramic::fork(|| {
            chan.send(&String::from("hello")).unwrap();
        });

        let s : String = chan.recv().unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn send_on_main_recv_in_subproc() {
        let chan   = ceramic::channel().unwrap();

        ceramic::fork(|| {
            chan.recv::<String>().unwrap();
        });
        chan.send(&String::from("hello")).unwrap();
        assert!(true);
    }

    #[test]
    fn ping_pong() {
        let chan   = ceramic::channel().unwrap();

        ceramic::fork(|| {
            chan.recv::<String>().unwrap();
            chan.send(&String::from("pong")).unwrap();
        });

        chan.send(&String::from("ping")).unwrap();
        let s : String = chan.recv().unwrap();
        assert_eq!(s, "pong");
    }
    #[test]
    fn two_forks_firewatch(){
        let chan  = ceramic::channel().unwrap();

        ceramic::fork(|| {
            chan.recv::<String>().unwrap();
        });

        ceramic::fork(|| {
            chan.recv::<String>().unwrap();
        });

        chan.send(&String::from("ping")).unwrap();
        chan.send(&String::from("ping")).unwrap();
        assert!(true);
    }

    #[test]
    fn close_read() {
        let chan   = ceramic::channel().unwrap();

        ceramic::fork(|| {
            chan.close().unwrap();
        });

        let s = chan.recv::<String>();
        assert_eq!(Err(ceramic::Error::ChannelClosed), s);
    }

    #[test]
    fn close_write() {
        let chan   = ceramic::channel().unwrap();

        ceramic::fork(|| {
            chan.close().unwrap();
        });

        let s = chan.send(&String::from("ping"));
        assert_eq!(Err(ceramic::Error::ChannelClosed), s);
    }

    #[test]
    #[should_panic]
    fn timout() {
        let chan   = ceramic::channel().unwrap();

        chan.set_timeout(Some(::std::time::Duration::new(0, 1000))).unwrap();
        chan.send(&String::from("ping")).unwrap();
    }
}
