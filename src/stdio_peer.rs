#[cfg(unix)]
extern crate tokio_file_unix;
#[cfg(unix)]
extern crate tokio_signal;

use std;
use std::thread;
use std::io::stdin;
use tokio_core::reactor::{Core, Handle};
use futures;
use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use tokio_io::{self,AsyncRead,AsyncWrite};
use std::io::{Read,Write};
use std::io::Result as IoResult;

use std::rc::Rc;
use std::cell::RefCell;

use futures::Async::{Ready, NotReady};

use tokio_io::io::copy;

use tokio_io::codec::FramedRead;
use std::fs::File;


#[cfg(unix)]
use self::tokio_file_unix::{File as UnixFile, StdFile};
#[cfg(unix)]
use std::os::unix::io::FromRawFd;

use super::{Peer, io_other_error, brokenpipe, wouldblock, BoxedNewPeerFuture, Result};

fn get_stdio_peer_impl(handle: &Handle) -> Result<Peer> {
    let si;
    let so;
    
    #[cfg(any(not(unix),feature="no_unix_stdio"))]
    {
        si = tokio_stdin_stdout::stdin(0);
        so = tokio_stdin_stdout::stdout(0);
    }
    
    #[cfg(all(unix,not(feature="no_unix_stdio")))]
    {
        let stdin  = self::UnixFile::new_nb(std::io::stdin())?;
        let stdout = self::UnixFile::new_nb(std::io::stdout())?;
    
        si = stdin.into_reader(&handle)?;
        so = stdout.into_io(&handle)?;
        
        let ctrl_c = tokio_signal::ctrl_c(&handle).flatten_stream();
        let prog = ctrl_c.for_each(|()| {
            UnixFile::raw_new(std::io::stdin()).set_nonblocking(false);
            UnixFile::raw_new(std::io::stdout()).set_nonblocking(false);
            ::std::process::exit(0);
            Ok(())
        });
        handle.spawn(prog.map_err(|_|()));
    }
    Ok(Peer::new(si,so))
}

pub fn get_stdio_peer(handle: &Handle) -> BoxedNewPeerFuture {
    Box::new(futures::future::result(get_stdio_peer_impl(handle))) as BoxedNewPeerFuture
}

pub fn restore_blocking_status() {
    #[cfg(all(unix,not(feature="no_unix_stdio")))]
    {
        UnixFile::raw_new(std::io::stdin()).set_nonblocking(false);
        UnixFile::raw_new(std::io::stdout()).set_nonblocking(false);
    }
}