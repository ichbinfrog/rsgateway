use std::{io::{self, Cursor}, net::TcpStream};

use super::request::Request;

pub trait TryClone<T> {
    fn clone(&self) -> io::Result<T>;
}

impl TryClone<TcpStream> for TcpStream {
    fn clone(&self) -> io::Result<TcpStream> {
        return self.try_clone();
    }
}

impl TryClone<Cursor<String>> for Cursor<String> {
    fn clone(&self) -> io::Result<Cursor<String>> {
        io::Result::Ok(Clone::clone(&self))
    }
}