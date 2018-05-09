extern crate tokio_process;

use std;
use tokio_core::reactor::{Handle};
use futures;
use tokio_io::{AsyncRead,AsyncWrite};
use std::io::{Read,Write};
use std::io::Result as IoResult;

use std::rc::Rc;
use std::cell::RefCell;


use std::process::Command;

use self::tokio_process::{CommandExt,Child};

use super::{Peer, BoxedNewPeerFuture};
use super::{once,Specifier,ProgramState,PeerConstructor,Options};
use ::std::process::Stdio;


#[derive(Debug,Clone)]
pub struct ShC(pub String);
impl Specifier for ShC {
    fn construct(&self, h:&Handle, _: &mut ProgramState, _opts: &Options) -> PeerConstructor {
        let mut args = Command::new("sh");
        args.arg("-c").arg(self.0.clone());
        once(Box::new(futures::future::result(process_connect_peer(h, args))) as BoxedNewPeerFuture)
    }
    specifier_boilerplate!(noglobalstate singleconnect no_subspec typ=Other);
}

fn process_connect_peer(h:&Handle, mut cmd: Command) -> Result<Peer,Box<std::error::Error>> {
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    let child = cmd.spawn_async(h)?;
    let ph = ProcessPeer(Rc::new(RefCell::new(child)));
    Ok(Peer::new(ph.clone(), ph))
}

#[derive(Clone)]
struct ProcessPeer(Rc<RefCell<Child>>);

impl Read for ProcessPeer {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.0.borrow_mut().stdout().as_mut().expect("assertion failed 1425").read(buf)
    }
}

impl Write for ProcessPeer {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.0.borrow_mut().stdin().as_mut().expect("assertion failed 1425").write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.0.borrow_mut().stdin().as_mut().expect("assertion failed 1425").flush()
    }
}

impl AsyncRead for ProcessPeer {}

impl AsyncWrite for ProcessPeer {
    fn shutdown(&mut self) -> futures::Poll<(), std::io::Error> {
        self.0.borrow_mut().stdin().as_mut().expect("assertion failed 1425").shutdown()
    }
}
