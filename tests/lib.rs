#[cfg(test)]

extern crate ceramic;
extern crate rain;

mod tests {
    use ceramic;

    #[test]
    fn recv_on_main_send_in_subproc() {
        let chan   = ceramic::channel().unwrap();

        let _prc    = ceramic::fork(|| {
            chan.send(&String::from("hello")).unwrap();
        });

        let s : String = chan.recv().unwrap().unwrap_or(String::from("nothing"));
        assert_eq!(s, "hello");
    }

    #[test]
    fn send_on_main_recv_in_subproc() {
        let chan   = ceramic::channel().unwrap();

        let _prc    = ceramic::fork(|| {
            chan.recv().unwrap();
        });
        chan.send(&String::from("hello")).unwrap();
        assert!(true);
    }

    #[test]
    fn ping_pong() {
        let chan   = ceramic::channel().unwrap();

        let _prc    = ceramic::fork(|| {
            chan.recv().unwrap();
            chan.send(&String::from("pong")).unwrap();
        });

        chan.send(&String::from("ping")).unwrap();
        let s : String = chan.recv().unwrap().unwrap_or(String::from("nothing"));
        assert_eq!(s, "pong");
    }
    #[test]
    fn two_forks_firewatch(){
        let chan  = ceramic::channel().unwrap();

        let _prc    = ceramic::fork(|| {
            chan.recv().unwrap();
        });

        let _prc2   = ceramic::fork(|| {
            chan.recv().unwrap();
        });

        chan.send(&String::from("ping")).unwrap();
        chan.send(&String::from("ping")).unwrap();
        assert!(true);
    }

    #[test]
    fn close_read() {
        let chan = ceramic::channel::<String>().unwrap();

        let _prc    = ceramic::fork(|| {
            chan.close().unwrap();
        });

        let s = chan.recv().unwrap();
        assert_eq!(s, None);
    }

    #[test]
    fn close_write() {
        let chan   = ceramic::channel().unwrap();

        let _prc    = ceramic::fork(|| {
            chan.close().unwrap();
        });

        let s = chan.send(&String::from("ping")).unwrap();
        assert_eq!(s, ());
    }

    #[test]
    #[should_panic]
    fn timout() {
        let chan = ceramic::channel().unwrap();

        chan.set_timeout(Some(::std::time::Duration::new(0, 1000))).unwrap();
        chan.send(&String::from("ping")).unwrap();
    }

    #[test]
    fn read_iter() {
        let chan = ceramic::channel().unwrap();

        let _prc  = ceramic::fork(|| {
            chan.send(&String::from("herp")).unwrap();
            chan.send(&String::from("derp")).unwrap();
            chan.close().unwrap();
        });

        for s in chan {
            println!(">>{}<<", s.unwrap());
        }
        assert!(true);
    }

    #[test]
    fn timout_after_send() {
        let chan = ceramic::channel().unwrap();
        chan.set_timeout(Some(::std::time::Duration::new(1,0))).unwrap();

        let _prc = ceramic::fork(|| {
            chan.recv().unwrap();
            //send never happens
        });

        chan.send(&String::from("ping")).unwrap();
        assert_eq!(chan.recv().unwrap_or(None), None);
    }
}
