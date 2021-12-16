pub type NodeId = usize;
pub const NODE_ID_ROOT: NodeId = 0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Directory = 1,
    RegularFile,
}

pub struct DEntry {
    pub name: String,
    pub ntype: NodeType,
}

pub trait FileSystem {
    fn read_dir(&self, dir: NodeId, pos: usize) -> Result<Option<DEntry>, String>;
    fn create(&mut self, dir: NodeId, dent: &DEntry) -> Result<NodeId, String>;
    fn read(&self, file: NodeId, off: usize, data: &mut [u8]) -> Result<usize, String>;
    fn write(&mut self, file: NodeId, off: usize, data: &[u8]) -> Result<usize, String>;
}
