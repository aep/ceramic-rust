#[cfg(test)]

extern crate ceramic;
extern crate rain;

mod tests {
    use ceramic;
    use rain::Graph;

    #[test]
    fn recv_on_main_send_in_subproc() {
        let mut graph = Graph::new();

        let chan   = ceramic::Chan::new();

        ceramic::Proc::new(|| {
            chan.send(&String::from("hello"));
        });

        let s : String = chan.recv().unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn send_on_main_recv_in_subproc() {
        let chan   = ceramic::Chan::new();

        ceramic::Proc::new(|| {
            chan.recv::<String>().unwrap();
        });
        chan.send(&String::from("hello"));
        assert!(true);
    }

    #[test]
    fn ping_pong() {
        let chan   = ceramic::Chan::new();

        ceramic::Proc::new(|| {
            chan.recv::<String>().unwrap();
            chan.send(&String::from("pong"));
        });

        chan.send(&String::from("ping"));
        let s : String = chan.recv().unwrap();
        assert_eq!(s, "pong");
    }
}
