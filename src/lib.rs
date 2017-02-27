//! Tools for dealing with tricky error handling situations
//!
//! This crate provides the `ResultIterExt` trait, which provides
//! conveniences for converting collections of `Result<T>` types to
//! collections of `T`, while handling errors correctly.
//!
//! The basic way to use this library is to call `fail_fast_if_err()?`
//! on any `Iterator` of `Result`, to "unwrap" all the individual
//! elements, returning the first error encountered.
//!
//! Note that although this method chains off an iterator, and
//! produces a new iterator, it drains he former iterator before
//! returning, and requires temporary storage proportional to the
//! elements returned by the original iterator.
//!
//! # Examples
//!
//! Read a directory of files to `String`, stopping at the
//! first error and returning it if necessary.
//!
//! ```rust
//! extern crate result_iter;
//!
//! use result_iter::ResultIterExt;
//! use std::io::{self, Read, BufReader};
//! use std::fs::{self, File};
//!
//! fn run() -> Result<Vec<String>, io::Error> {
//!     // Read a directory of files into a Vec, where each
//!     // file my generate an error
//!     let maybe_strings;
//!     maybe_strings = fs::read_dir(".")?
//!         .map(|dirent| {
//!             dirent.and_then(|d| File::open(d.path()))
//!                 .and_then(|f| {
//!                     let mut f = BufReader::new(f);
//!                     let mut s = String::new();
//!                     f.read_to_string(&mut s)?;
//!             Ok(s)})
//!         });
//!
//!     // As soon as we encounter an error, return it.
//!     // Otherwise return a Vec<String>
//!     let strings = maybe_strings.fail_fast_if_err()?.collect();
//!     Ok(strings)
//! }
//!
//! fn main() {
//!     let _ = run();
//! }
//! ```
//!
//! Read a directory of files to `String`, continuing after
//! the first error, and returning all errors.
//!
//! ```rust
//! extern crate result_iter;
//!
//! use result_iter::{ResultIterExt, MultiError};
//! use std::io::{self, Read, BufReader};
//! use std::fs::{self, File};
//!
//! fn run() -> Result<Vec<String>, MultiError<io::Error>> {
//!     // Read a directory of files into a Vec, where each
//!     // file my generate an error
//!     let maybe_strings;
//!     maybe_strings = fs::read_dir(".")
//!         // Map io::Error to MultiError<io::Error> to satisfy
//!         // the example return type.
//!         .map_err(|e| MultiError::new(vec![e]))?
//!         .map(|dirent| {
//!             dirent.and_then(|d| File::open(d.path()))
//!                 .and_then(|f| {
//!                     let mut f = BufReader::new(f);
//!                     let mut s = String::new();
//!                     f.read_to_string(&mut s)?;
//!             Ok(s)})
//!         });
//!
//!     let maybe_strings = maybe_strings.collect::<Vec<_>>();
//!     let maybe_strings = maybe_strings.into_iter();
//!
//!     // As soon as we encounter an error, return it.
//!     // Otherwise return a Vec<String>
//!     let strings = maybe_strings.fail_slow_if_err()?.collect();
//!     Ok(strings)
//! }
//!
//! fn main() {
//!     let _ = run();
//! }
//! ```

use std::error::Error as StdError;
use std::fmt;
use std::vec;

pub trait ResultIterExt<T, E>: Sized + Iterator<Item = Result<T, E>> {
    fn end_if_err(self) -> EndIfErrIter<T, E, Self>;
    fn fail_slow_if_err(self) -> Result<vec::IntoIter<T>, MultiError<E>>;

    fn fail_fast_if_err(self) -> Result<vec::IntoIter<T>, E> {
        self.end_if_err().fail_slow_if_err()
            .map_err(|e| e.into_iter().next().expect(""))
    }
}

impl<T, E, I> ResultIterExt<T, E> for I
    where I: Iterator<Item = Result<T, E>>
{
    fn end_if_err(self) -> EndIfErrIter<T, E, Self> {
        EndIfErrIter(self, State::Continue)
    }

    fn fail_slow_if_err(self) -> Result<vec::IntoIter<T>, MultiError<E>> {
        let mut goodies = vec![];
        let mut baddies = vec![];

        let mut still_ok = true;

        for el in self {
            match el {
                Ok(a) => {
                    // Don't keep pushing goodies once we've errored
                    if !still_ok { continue }

                    goodies.push(a);
                }
                Err(b) => {
                    still_ok = false;

                    baddies.push(b);
                }
            }
        }

        if baddies.is_empty() {
            Ok(goodies.into_iter())
        } else {
            Err(MultiError::new(baddies))
        }
    }
}

enum State { Continue, End }

pub struct EndIfErrIter<T, E, I>(I, State)
    where I: Iterator<Item = Result<T, E>>;

impl<T, E, I> Iterator for EndIfErrIter<T, E, I>
    where I: Iterator<Item = Result<T, E>>
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.1 {
            State::Continue => {
                match self.0.next() {
                    Some(n) => {
                        if n.is_err() {
                            self.1 = State::End;
                        }

                        Some(n)
                    }
                    None => None
                }
            }
            State::End => {
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct MultiError<E>(Vec<E>);

impl<E> StdError for MultiError<E>
    where E: StdError
{
    fn description(&self) -> &str { self.0[0].description() }
}

impl<E> MultiError<E> {
    pub fn new(errors: Vec<E>) -> MultiError<E> { MultiError(errors) }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn into_iter(self) -> vec::IntoIter<E> { self.0.into_iter() }
}

impl<E> fmt::Display for MultiError<E>
    where E: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0[0], f)
    }
}
