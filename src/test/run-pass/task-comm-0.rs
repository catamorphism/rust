extern mod std;

use pipes::Chan;
use pipes::Port;

fn main() { test05(); }

fn test05_start(ch : Chan<int>) {
    ch.send(10);
    error!("sent 10");
    ch.send(20);
    error!("sent 20");
    ch.send(30);
    error!("sent 30");
}

fn test05() {
    let (ch, po) = pipes::stream();
    task::spawn(|move ch| test05_start(ch) );
    let mut value = po.recv();
    log(error, value);
    value = po.recv();
    log(error, value);
    value = po.recv();
    log(error, value);
    assert (value == 30);
}
