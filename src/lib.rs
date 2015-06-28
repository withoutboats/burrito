//! `Burrito` is a monadic IO interface, similar to Haskell's IO monad.
//!
//! A single `Burrito` type wraps a single IO handle, whether its the stdio, a file on the system,
//! or a socket address. There are several ways to create a `Burrito`.
//!
//! * `burrito()` creates an stdio `Burrito.`
//! * `Burrito::wrap()` wraps a Result<T>, where T is an IO handle.
//! * `Burrito::wrap_func()` wraps a function which returns a Result<T>, where T is an IO handle.
//! * Types which implement `FromPath` can be wrapped using `Burrito::from_path()`
//! * Types which implement `FromAddr` can be wrapped using `Burrito::from_addr()`
//!
//! IO actions can be performed directly on the `Burrito`; the result of these actions can be
//! accessed using the `and_then` method. IO failure can be handled using the `or_else` method.
//!
//! As a simple example of a `Burrito` in action, this code will echo stdin on stdout.
//!
//! ```no_run
//! # extern crate burrito;
//! # fn main() {
//! use burrito::burrito;
//!
//! burrito().read_line().and_then(|echo, burrito| burrito.print_line(&echo));
//! # }
//! ```
//!
//! # The `RealWorld` type
//!
//! The default `Burrito`, which is returned by `burrito()`, wraps a type called `RealWorld`. This
//! encapsulates the handles to stdin, stdout, stderr. It does not lock these handles; it will
//! attempt to lock them each time they are written to / read from.
//!
//! The `RealWorld` type is not directly accessible except wrapped inside of the `Burrito` returned
//! by `burrito()`.
//!
//! # Using the monadic interface
//!
//! Every `Burrito` has two type parameters. The `T` type is the type of the IO handle it wraps.
//! The `A` type, however, is the type of the data returned by the most recent IO call.
//!
//! All of the IO calls implemented on `Burrito` consume the object and return a new `Burrito`; if
//! that call returned data of some kind, that data is stored inside that new `Burrito`.
//!
//! ## `and_then()`
//!
//! The prefered way to access the data inside a burrito is the `and_then` method. `and_thenn`
//! takes a function of `A` and `Burrito<(), T>` to any `Burrito`, and applies that function to the
//! `A` data inside the `Burrito` and a new `Burrito` wrapped around the IO handle.
//!
//! A common pattern when using `Burrito`s is a back and forth between IO methods and `and_then`
//! calls, possibly including IO methods on the burrito and the end of the `and_then`.
//!
//! ```
//! # extern crate burrito;
//! # fn main() {
//! use burrito::burrito;
//!
//! burrito().read_line().and_then(|input, burrito| {
//!     let output = // Generate output based on the input in some way.
//! # &input;
//!     burrito.print_line(output)
//! }).read_line().and_then(|input, burrito| {
//!     // Do more stuff with the next input.
//!     burrito
//! });
//! # }
//! ```
//! 
//! Note that the function passed to `and_then` is only called in the event that the `Burrito` is
//! not in a state of failure.
//!
//! ## `and()`
//!
//! The `and()` method is less powerful than the `and_then()` method because it does not provide
//! access to the data inside the `Burrito`. Its main use is to close this IO handle when its done
//! being used and to open another one, which could possibly be of a different type. It is not
//! lazy, and the IO handle will actually be opened even if the `Burrito` is in a state of failure.
//!
//! ## `or_else()`
//!
//! The `or_else()` method provides access to the inner error in the event that the `Burrito`
//! represents an IO handle in a state of failure. As soon as any IO method on that handle returns
//! a failure, the `Burrito` remains in a state of failure until re-instantiated. This method must
//! either create a `Burrito` of the same type (presumably replacing the original `Burrito`) or
//! else diverge, e.g. by exiting or panicking.
//!
//! ## `or()`
//!
//! The `or()` method enables replacing the `Burrito` with another of the same types, but does not
//! provide access to the inner error. It is not lazy, and will actually open the handle even if
//! the `Burrito` is not in a state of failure.

use std::convert::AsRef;
use std::default::Default;
use std::fmt;
use std::path::Path;
use std::io::{self, Read, Write, Seek, BufRead};
use std::net::ToSocketAddrs;

mod realworld;
mod iomonad;
mod constructors;

use realworld::RealWorld;
use iomonad::Io;
use iomonad::Io::*;
pub use constructors::{FromPath, FromAddr};

/// Create a default burrito (wrapping the stdio handles).
pub fn burrito() -> Burrito<(), RealWorld> { Burrito::default() }

/// The fundamental monadic type of the burrito library.
///
/// `Burrito` implements different IO methods depending on the traits implemented by the IO handle
/// it wraps. These methods have the same name as the methods associated with that trait, though
/// their signature differs somewhat.
pub struct Burrito<A, T>(Io<A, T>);

/// These methods construct `Burrito`s of arbitrary types.
impl<T> Burrito<(), T> {

    /// The basic constructor for `Burrito`. This takes an `io::Result<T>`, where `T` is the type
    /// being wrapped by the `Burrito`, `io::Result<T>` is the return type of the constructor for
    /// most IO handle types.
    ///
    /// ```
    /// # extern crate burrito;
    /// # fn main() {
    /// use std::fs::File;
    /// use burrito::Burrito;
    ///
    /// let burrito = Burrito::wrap(File::create("/foo/bar/baz"));
    /// # }
    /// ```
    pub fn wrap(inner: io::Result<T>) -> Burrito<(), T> {
        match inner {
            Ok(io) => Burrito(Good((), io)),
            Err(err) => Burrito(Bad(err)),
        }
    }

    /// A constructor for `Burrito` which takes a function and wraps the result of that function.
    ///
    /// ```
    /// # extern crate burrito;
    /// # fn main() {
    /// use std::fs::{self, File};
    /// use burrito::Burrito;
    ///
    /// let burrito = Burrito::wrap_func(|| {
    ///     try!(fs::metadata("/foo/bar/baz"));
    ///     File::open("/foo/bar/baz")
    /// });
    /// # }
    /// ```
    pub fn wrap_func<F: FnOnce() -> io::Result<T>>(f: F) -> Burrito<(), T> {
        match f() {
            Ok(io) => Burrito(Good((), io)),
            Err(err) => Burrito(Bad(err)),
        }
    }

}

/// These two functions are constructors for types which can be constructed from paths and socket
/// addresses.
impl Burrito<(), ()> {

    /// Constructs an IO handle using the path argument, according to that IO handle's
    /// implementation of FromPath, then wraps that handle in a `Burrito`. It is a good idea to
    /// type annotate this call to ensure the correct kind of handle is constructed.
    ///
    /// ```rust
    /// # extern crate burrito;
    /// # fn main() {
    /// use std::fs::File;
    /// use burrito::{Burrito, FromPath};
    ///
    /// let burrito = Burrito::from_path::<_, File>("/foo/bar/baz");
    /// # }
    /// ```
    pub fn from_path<P: AsRef<Path>, T: FromPath>(path: P) -> Burrito<(), T> {
        match T::from_path(path) {
            Ok(io) => Burrito(Good((), io)),
            Err(err) => Burrito(Bad(err)),
        }
    }

    /// Constructs an IO handle using the addr argument, according to that IO handle's
    /// implementation of FromAddr, then wraps that handle in a `Burrito`. It is a good idea to
    /// type annotate this call to ensure the correct kind of handle is constructed.
    ///
    /// ```rust
    /// # extern crate burrito;
    /// # fn main() {
    /// use std::net::TcpStream;
    /// use burrito::{Burrito, FromAddr};
    ///
    /// let burrito = Burrito::from_addr::<_, TcpStream>("localhost:12345");
    /// # }
    /// ```
    pub fn from_addr<A: ToSocketAddrs, T: FromAddr>(addr: A) -> Burrito<(), T> {
        match T::from_addr(addr) {
            Ok(io) => Burrito(Good((), io)),
            Err(err) => Burrito(Bad(err)),
        }
    }

}

/// These methods are defined for all `Burrito`s.
impl<A, T> Burrito<A, T> {

    /// Allows you to 'pivot' to a new `Burrito` if this one is good, or to remain in a state of
    /// failure if this `Burrito` has failed. See the module level documentation for more info.
    pub fn and<B, U>(self, alternative: Burrito<B, U>) -> Burrito<B, U> {
        match self {
            Burrito(Good(..)) => alternative,
            Burrito(Bad(err)) => Burrito(Bad(err)),
        }
    }

    /// Allows access to data returned by the most recent IO call on this `Burrito`; this function
    /// must return another `Burrito` of some kind or else diverge. See the module level
    /// documentation for more info.
    pub fn and_then<B, U, F>(self, f: F) -> Burrito<B, U>
            where F: FnOnce(A, Burrito<(), T>) -> Burrito<B, U> {
        match self {
            Burrito(Good(data, io)) => f(data, Burrito(Good((), io))),
            Burrito(Bad(err)) => Burrito(Bad(err))
        }
    }

    /// Allows you to substitute this `Burrito` for another of the same type if it has gone bad.
    pub fn or(self, alternative: Burrito<A, T>) -> Burrito<A, T> {
        match self {
            Burrito(Bad(..)) => alternative,
            _ => self,
        }
    }

    /// Allows access to the error thrown if this `Burrito` has gone bad. This function must return
    /// another `Burrito` of the same type or else diverge. See the module level documentation for
    /// more info.
    pub fn or_else<F>(self, f: F) -> Burrito<A, T> 
            where F: FnOnce(io::Error) -> Burrito<A, T> {
        match self {
            Burrito(Bad(err)) => f(err),
            _ => self
        }
    }

    /// Drops any data returned by the most recent IO call.
    pub fn ignore(self) -> Burrito<(), T> {
        match self {
            Burrito(Good(_, io)) => Burrito(Good((), io)),
            Burrito(Bad(err)) => Burrito(Bad(err))
        }
    }

    /// Returns true if the `Burrito` has not failed.
    pub fn is_good(&self) -> bool {
        match *self {
            Burrito(Good(..)) => true,
            Burrito(Bad(..)) => false,
        }
    }

    /// Returns true if the `Burrito` has failed.
    pub fn is_bad(&self) -> bool { !self.is_good() }

    /// Converts the `Burrito` to a `Result` of both the handle and the most recently returned
    /// data.
    pub fn ok(self) -> io::Result<(A, T)> {
        match self {
            Burrito(Good(data, io)) => Ok((data, io)),
            Burrito(Bad(err)) => Err(err),
        }
    }

    /// Converts the `Burrito` to a `Result` of the most recently returned data.
    pub fn to_data(self) -> io::Result<A> {
        match self {
            Burrito(Good(data, _)) => Ok(data),
            Burrito(Bad(err)) => Err(err),
        }
    }

    /// Converts the `Burrito` to a `Result` of the IO handle wrapped within.
    pub fn to_handle(self) -> io::Result<T> {
        match self {
            Burrito(Good(_, io)) => Ok(io),
            Burrito(Bad(err)) => Err(err),
        }
    }

}

impl Default for Burrito<(), RealWorld> {
    fn default() -> Burrito<(), RealWorld> { Burrito(Good((), RealWorld::default())) }
}

impl<A, T> Burrito<A, T> where T: Read {
    /// Performs a read on the IO handle inside the burrito. Will read into a buffer of _n_ bytes.
    ///
    /// Though the buffer passed to `Read::read()` can be stack allocated, this function allocates
    /// the buffer on the heap, so that its length can be determined by the function call. The
    /// `Vec<u8>` returned by this type will contain all of the bytes read from the call; if that
    /// is less than _n_, it will not include any null bytes.
    pub fn read(self, n: usize) -> Burrito<Vec<u8>, T> { Burrito(self.0.read(n)) }
    /// Reads to the end of the handle inside the burrito, returning a `Vec<u8>` of bytes.
    pub fn read_to_end(self) -> Burrito<Vec<u8>, T> { Burrito(self.0.read_to_end()) }
    /// Reads everything from the handle into a `String`.
    pub fn read_to_string(self) -> Burrito<String, T> { Burrito(self.0.read_to_string()) }
}

impl<A, T> Burrito<A, T> where T: Write {
    /// Writes the content of the buf to the IO handle; returns the number of bytes written.
    pub fn write(self, buf: &[u8]) -> Burrito<usize, T> { Burrito(self.0.write(buf)) }
    /// Writes the content of the buf to the IO handle; will write all of the bytes unless it
    /// fails.
    pub fn write_all(self, buf: &[u8]) -> Burrito<(), T> { Burrito(self.0.write_all(buf)) }
    /// Writes formatted text to the IO handle.
    pub fn write_fmt(self, buf: fmt::Arguments) -> Burrito<(), T> {
        Burrito(self.0.write_fmt(buf))
    }
}

impl<A, T> Burrito<A, T> where T: Seek {
    /// Seeks to a position in the IO handle; returns the actual position that has been `seek`ed
    /// to.
    pub fn seek(self, pos: io::SeekFrom) -> Burrito<u64, T> { Burrito(self.0.seek(pos)) }
}

impl<A, T> Burrito<A, T> where T: BufRead {
    /// Fills the buffer on the buffered reader. Unlike the underlying fill_buf macro, this does
    /// not return a reference to the bytes in the buffer.
    pub fn fill_buf(self) -> Burrito<(), T> { Burrito(self.0.fill_buf()) }
    /// Marks `amt` bytes in the buffer as consumed.
    pub fn consume(self, amt: usize) -> Burrito<(), T> { Burrito(self.0.consume(amt)) }
    /// Reads from the buffered reader until the `byte` is reached.
    pub fn read_until(self, byte: u8) -> Burrito<Vec<u8>, T> { Burrito(self.0.read_until(byte)) }
    /// Reads a line from the buffered reader.
    pub fn read_line(self) -> Burrito<String, T> { Burrito(self.0.read_line()) }
    /// Generates a Split Iterator of the underlying buffered reader. This will be wrapped in a
    /// result because the IO handle may have failed at some point in the past.
    pub fn split(self, byte: u8) -> io::Result<io::Split<T>> { self.0.split(byte) }
    /// Generates a Lines Iterator of the underlying buffered reader. This will be wrapped in a
    /// result because the IO handle may have failed at some point in the past.
    pub fn lines(self) -> io::Result<io::Lines<T>> { self.0.lines() }
}

/// These methods are implemented only for the stdio `Burrito`. Note that `RealWorld` implements
/// both `Read` and `Write`, and so the stdio `Burrito` also has all methods for `Burrito`s
/// wrapping handles which implement those traits; the methods associated with the `Write` trait
/// write to stdout, whereas a set of special `to_err()` methods write to stderr.
impl<A> Burrito<A, RealWorld> {

    /// Prints a string to stdout, with a newline affixed to the end. Internally, it calls
    /// `write_all`; to use it like the `println!()` macro, you can use a reference to a format 
    /// macro - that is `&format!()`.
    ///
    /// ```
    /// # extern crate burrito;
    /// # fn main() {
    /// use burrito::burrito;
    ///
    /// burrito().print_line(&format!("2 + 2 = {}", 4));
    /// # }
    /// ```
    pub fn print_line(self, buf: &str) -> Burrito<(), RealWorld> {
        Burrito(self.0.print_line(buf))
    }

    /// Reads a line from stdin. This has the same behavior as the read_line() method on io::Stdin.
    /// 
    /// ```no_run
    /// # extern crate burrito;
    /// # fn main() {
    /// use burrito::burrito;
    /// 
    /// let input = burrito().read_line().to_data();
    /// # }
    /// ```
    pub fn read_line(self) -> Burrito<String, RealWorld> {
        Burrito(self.0.read_line())
    }

    /// Performs a write to stderr instead of stdout.
    pub fn write_to_err(self, buf: &[u8]) -> Burrito<usize, RealWorld> {
        Burrito(self.0.write_to_err(buf))
    }

    /// Performs a write_all to stderr instead of stdout.
    pub fn write_all_to_err(self, buf: &[u8]) -> Burrito<(), RealWorld> {
        Burrito(self.0.write_all_to_err(buf))
    }

    /// Performs a write_fmt to stderr instead of stdout.
    pub fn write_fmt_to_err(self, fmt: fmt::Arguments) -> Burrito<(), RealWorld> {
        Burrito(self.0.write_fmt_to_err(fmt))
    }

}
