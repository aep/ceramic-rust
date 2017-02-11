[![Build Status](https://travis-ci.org/aep/ceramic-rust.svg?branch=master)](https://travis-ci.org/aep/ceramic-rust)
[![codecov](https://codecov.io/gh/aep/ceramic-rust/branch/master/graph/badge.svg)](https://codecov.io/gh/aep/ceramic-rust)
[![crates.io](http://meritbadge.herokuapp.com/ceramic)](https://crates.io/crates/ceramic)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)
[![docs](https://docs.rs/ceramic/badge.svg)](https://docs.rs/ceramic)


Synchronous channels for rust between posix proccesses
-------------------------------------------------


Ceramic is a simple and effective way to isolate crappy code into processes. The code gets its own heap and
can be terminated without affecting the main process.

It fullfills the same use case as servo/ipc-channel, but with a much more consistent and simple design.
The downside is that it propbably doesn't work on windows.

This is a rust port of the original C++ library [https://github.com/aep/ceramic](https://github.com/aep/ceramic).

Serialize and Deserialize traits are required for all types passed over the channel.


```rust
use ceramic;

fn main() {
    let chan = ceramic::channel().unwrap();

    let _p = ceramic::fork(|| {
        chan.send(&String::from("hello")).unwrap();
    });

    chan.set_timeout(Some(::std::time::Duration::new(1,0))).unwrap();
    let s : String = chan.recv().unwrap_or(Nothing).unwrap_or(String::from("nothing"));
    println!("{}", s);
}
```


or use it as iterator:

```rust
fn main() {
   let chan = ceramic::channel().unwrap();

   let _p = ceramic::fork(|| {
       chan.send(&String::from("herp")).unwrap();
       chan.send(&String::from("derp")).unwrap();
       chan.close().unwrap();
   });

   for s in chan {
       println!(">>{}<<", s.unwrap());
   }
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
- correct close write semantics are unclear
- tests with pthread
- deadlock detection
- test on osx

