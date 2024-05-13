use std::{io, net::TcpStream};

pub trait TryClone<T> {
    fn clone(&self) -> io::Result<T>;
}

impl TryClone<TcpStream> for TcpStream {
    fn clone(&self) -> io::Result<TcpStream> {
        return self.try_clone();
    }
}
