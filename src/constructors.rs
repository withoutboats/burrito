use std::convert::AsRef;
use std::fs;
use std::net::{self, ToSocketAddrs};
use std::path::Path;
use std::io;

pub trait FromPath {
    fn from_path<P: AsRef<Path>>(P) -> io::Result<Self>;
}

pub trait FromAddr {
    fn from_addr<A: ToSocketAddrs>(A) -> io::Result<Self>;
}

impl FromPath for fs::File {
    fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        fs::OpenOptions::new().read(true).write(true).create(true).open(path)
    }
}

impl FromAddr for net::TcpStream {
    fn from_addr<A: ToSocketAddrs>(addr: A) -> io::Result<net::TcpStream> {
        net::TcpStream::connect(addr)
    }
}

