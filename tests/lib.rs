#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]
#[cfg(test)]

extern crate ceramic;

mod tests {

    use ceramic;

    #[test]
    fn it_works() {
        let chan   = ceramic::Chan::new();
        let thread = ceramic::Proc::new(|| {
            println!("child");
            chan.send(String::from("hello").as_bytes());
        });

        let mut buf = [0;128];
        chan.recv(&mut buf);

        println!("parent received: {}", String::from_utf8_lossy(&buf));
    }
}
