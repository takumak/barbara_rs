extern crate posix;

#[derive(Debug)]
pub struct ElfParserError {
    errno: posix::Errno,
    message: String,
}

impl ElfParserError {
    pub fn new(errno: posix::Errno, message: String) -> Self {
        Self {
            errno,
            message,
        }
    }
}
