extern crate posix;

#[derive(PartialEq, Debug)]
pub struct ElfParserError {
    errno: posix::Errno,
    message: String,
}

impl ElfParserError {
    pub(crate) fn new(errno: posix::Errno, message: String) -> Self {
        Self { errno, message }
    }
}

#[cfg(test)]
mod tests {
    use crate::ElfParserError;

    #[test]
    fn elfparsererror_partialeq() {
        let err1 = ElfParserError::new(posix::Errno::EINVAL, format!("Test"));
        let err2 = ElfParserError::new(posix::Errno::EINVAL, format!("Test"));
        assert_eq!(err1, err2);
    }

    #[test]
    fn elfparsererror_debug() {
        let err = ElfParserError::new(posix::Errno::EINVAL, format!("Test"));
        assert_eq!(
            format!("{:?}", err),
            "ElfParserError { errno: EINVAL, message: \"Test\" }"
        );
    }
}
