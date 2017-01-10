[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)
[![crates.io](http://meritbadge.herokuapp.com/ceramic)](https://crates.io/crates/ceramic)
[![docs](https://docs.rs/ceramic/badge.svg)](https://docs.rs/ceramic)


Synchronous channels for rust between proccesses
-------------------------------------------------

This is a rust port of [https://github.com/aep/ceramic](https://github.com/aep/ceramic).

I'm just starting to learn rust, so this is not production ready code yet.

Ceramic is a simple and effective way to isolate rust code into processes.

A famous example of terrible api is gethostbyname.
POSIX actually removed it, because it's shit, but alot of unix code still uses it,
and so we have to deal with library calls that simply never terminate.


```rust
use ceramic;

fn main() {
    let chan   = ceramic::Chan::new();
    let thread = ceramic::Proc::new(|| {
        chan.send(String::from("hello").as_bytes());
    });

    let mut buf = [0;128];
    chan.recv(&mut buf);
    println!("parent received: {}", String::from_utf8_lossy(&buf));
}
```

channel synchronizing behaviour
-------------------------------

send and receive operations _must be_ symetric.
I.e. a read() will block until write() is called from another thread, but write() will also block until there is a read().
This is essentially how golang's chan(0) behaves, and _not_ how unix IPC usually behaves.


This makes it easy to reason about code that uses ceramic, because it introduces synchronization points.


```C
primes = chan();
thread() {
    do {
        int nr = heavyMath();
    } while (chan << nr)
}

primes >> nr;
primes >> nr;
primes >> nr;
primes.close();

```



However, it also introduces possibilities for new race condition that would not exist with buffered channels, for example this is invalid:


```C
thread() {
    write();
}
thread() {
    read();
}
write();
read();

```

this might look like:
- fork some threads A and B which wait
- then write to thread B
- and  read from thread A

but randomly the OS scheduler might decide on this order:

- fork some threads A, B which wait
- A writes to B
- main thread deadlocks on write



There is an argument in golang, that you should use buffered channels only when the code works unbuffered,
to avoid exactly these situations where something only doesn't deadlock because of buffers.
but to a reader the codeflow is still entirely unclear.

TODO
----

- api to clean up unused sockets
- tests for close
- close doesnt stop write yet
- tests with pthread
- deadlock detection
- use more efficient streaming
- test on osx
- automate tests + coverage

