use crate::posix;

pub type NodeId = usize;
pub const NODE_ID_ROOT: NodeId = 0;

#[derive(Debug)]
pub struct FsError {
    errno: posix::Errno,
    message: String,
}

impl FsError {
    pub fn new(errno: posix::Errno, message: String) -> Self {
        Self { errno, message }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeType {
    Directory = 1,
    RegularFile,
}

#[derive(Debug)]
pub struct DEntry {
    pub name: String,
    pub ntype: NodeType,
}

pub trait FileSystem {
    fn readdir(&self, dir: NodeId, pos: usize) -> Result<Option<(DEntry, NodeId)>, FsError>;
    fn create(&mut self, dir: NodeId, dent: &DEntry) -> Result<NodeId, FsError>;
    fn read(&self, file: NodeId, off: usize, data: &mut [u8]) -> Result<usize, FsError>;
    fn write(&mut self, file: NodeId, off: usize, data: &[u8]) -> Result<usize, FsError>;
    fn truncate(&mut self, file: NodeId, len: usize) -> Result<(), FsError>;
    fn getsize(&self, file: NodeId) -> Result<usize, FsError>;

    fn lookup(&self, dir: NodeId, name: &str) -> Result<Option<NodeId>, FsError> {
        let mut pos: usize = 0;
        loop {
            match self.readdir(dir, pos)? {
                Some((ent, node_id)) => {
                    if ent.name == *name {
                        return Ok(Some(node_id));
                    }
                }
                None => return Ok(None),
            }
            pos += 1
        }
    }

    fn lookup_path(&self, path: &[&str]) -> Result<Option<NodeId>, FsError> {
        let mut dir;
        let mut file = NODE_ID_ROOT;
        for name in path {
            dir = file;
            file = match self.lookup(dir, name)? {
                Some(node_id) => node_id,
                None => return Ok(None),
            };
        }
        return Ok(Some(file));
    }
}

#[cfg(test)]
mod tests {
    use crate::posix;
    use crate::{DEntry, FsError, NodeType};

    #[test]
    fn fserror_debug() {
        let err = FsError::new(posix::Errno::EINVAL, String::from("test error"));
        assert_eq!(
            format!("{:?}", err),
            "FsError { errno: EINVAL, message: \"test error\" }"
        );
    }

    #[test]
    fn dentry_debug() {
        let dent = DEntry {
            name: String::from("test_file"),
            ntype: NodeType::RegularFile,
        };
        assert_eq!(
            format!("{:?}", dent),
            "DEntry { name: \"test_file\", ntype: RegularFile }"
        );
    }
}
