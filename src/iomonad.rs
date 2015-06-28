use std::fmt;
use std::io::{self, Read, Write, Seek, BufRead};

use realworld::RealWorld;
use self::Io::*;

pub enum Io<A, T> {
    Good(A, T),
    Bad(io::Error),
}

impl<A, T> Io<A, T> where T: Read {

    pub fn read(self, n: usize) -> Io<Vec<u8>, T> {
        match self {
            Good(_, mut r) => {
                let mut buf = vec![0; n];
                match r.read(&mut buf) {
                    Ok(n) => {
                        buf.truncate(n);
                        Good(buf, r)
                    }
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn read_to_end(self) -> Io<Vec<u8>, T> {
        match self {
            Good(_, mut r) => {
                let mut buf = Vec::new(); 
                match r.read_to_end(&mut buf) {
                    Ok(..) => Good(buf, r),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn read_to_string(self) -> Io<String, T> {
        match self {
            Good(_, mut r) => {
                let mut buf = String::new();
                match r.read_to_string(&mut buf) {
                    Ok(..) => Good(buf, r),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

}

impl<A, T> Io<A, T> where T: Write {
    
    /// Write from inside the burrito.
    pub fn write(self, buf: &[u8]) -> Io<usize, T> {
        match self {
            Good(_, mut w) => {
                match w.write(buf) {
                    Ok(n) => Good(n, w),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        } 
    }

    pub fn write_all(self, buf: &[u8]) -> Io<(), T> {
        match self {
            Good(_, mut w) => {
                match w.write_all(buf) {
                    Ok(..) => Good((), w),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn write_fmt(self, fmt: fmt::Arguments) -> Io<(), T> {
        match self {
            Good(_, mut w) => {
                match w.write_fmt(fmt) {
                    Ok(..) => Good((), w), 
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

}

impl<A, T> Io<A, T> where T: Seek {

    pub fn seek(self, pos: io::SeekFrom) -> Io<u64, T> {
        match self {
            Good(_, mut s) => {
                match s.seek(pos) {
                    Ok(n) => Good(n, s),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

}

impl<A, T> Io<A, T> where T: BufRead {

    pub fn fill_buf(self) -> Io<(), T> {
        match self {
            Good(_, mut r) => {
                match r.fill_buf() {
                    Ok(..) => Good((), r),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn consume(self, amt: usize) -> Io<(), T> {
        match self {
            Good(_, mut r) => {
                r.consume(amt);
                Good((), r)
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn read_until(self, byte: u8) -> Io<Vec<u8>, T> {
        match self {
            Good(_, mut r) => {
                let mut buf = Vec::new();
                match r.read_until(byte, &mut buf) {
                    Ok(..) => Good(buf, r),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn read_line(self) -> Io<String, T> {
        match self {
            Good(_, mut r) => {
                let mut buf = String::new();
                match r.read_line(&mut buf) {
                    Ok(..) => Good(buf, r),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn split(self, byte: u8) -> io::Result<io::Split<T>> {
        match self {
            Good(_, r) => Ok(r.split(byte)),
            Bad(err) => Err(err)
        }
    }

    pub fn lines(self) -> io::Result<io::Lines<T>> {
        match self {
            Good(_, r) => Ok(r.lines()),
            Bad(err) => Err(err),
        }
    }

}

impl<A> Io<A, RealWorld> {

    pub fn print_line(self, buf: &str) -> Io<(), RealWorld> {
        match self {
            Good(_, rw) => {
                let result = rw.stdout.lock().write_all(format!("{}\n", buf).as_bytes());
                match result {
                    Ok(..) => Good((), rw),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn read_line(self) -> Io<String, RealWorld> {
        match self {
            Good(_, mut rw) => {
                let mut buf = String::new();
                match rw.stdin.read_line(&mut buf) {
                    Ok(..) => Good(buf, rw),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn write_to_err(self, buf: &[u8]) -> Io<usize, RealWorld> {
        match self {
            Good(_, rw) => {
                let result = rw.stderr.lock().write(buf);
                match result {
                    Ok(n) => Good(n, rw),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn write_all_to_err(self, buf: &[u8]) -> Io<(), RealWorld> {
        match self {
            Good(_, rw) => {
                let result = rw.stderr.lock().write_all(buf);
                match result {
                    Ok(..) => Good((), rw),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

    pub fn write_fmt_to_err(self, fmt: fmt::Arguments) -> Io<(), RealWorld> {
        match self {
            Good(_, rw) => {
                let result = rw.stderr.lock().write_fmt(fmt);
                match result {
                    Ok(..) => Good((), rw),
                    Err(err) => Bad(err),
                }
            }
            Bad(err) => Bad(err)
        }
    }

}
