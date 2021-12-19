#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Errno {
    EPERM   =  1,
    ENOENT  =  2,
    ESRCH   =  3,
    EINTR   =  4,
    EIO     =  5,
    ENXIO   =  6,
    E2BIG   =  7,
    ENOEXEC =  8,
    EBADF   =  9,
    ECHILD  = 10,
    EAGAIN  = 11,
    ENOMEM  = 12,
    EACCES  = 13,
    EFAULT  = 14,
    ENOTBLK = 15,
    EBUSY   = 16,
    EEXIST  = 17,
    EXDEV   = 18,
    ENODEV  = 19,
    ENOTDIR = 20,
    EISDIR  = 21,
    EINVAL  = 22,
    ENFILE  = 23,
    EMFILE  = 24,
    ENOTTY  = 25,
    ETXTBSY = 26,
    EFBIG   = 27,
    ENOSPC  = 28,
    ESPIPE  = 29,
    EROFS   = 30,
    EMLINK  = 31,
    EPIPE   = 32,
    EDOM    = 33,
    ERANGE  = 34,
}



#[cfg(test)]
mod tests {
    use crate::Errno;

    #[test]
    fn errno_clone() {
        assert_eq!(Errno::EINVAL, Errno::EINVAL.clone());
    }

    #[test]
    fn errno_debug() {
        assert_eq!(format!("{:?}", Errno::EINVAL), "EINVAL");
    }
}
