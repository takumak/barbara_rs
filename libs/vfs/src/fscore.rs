#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Directory = 1,
    RegularFile,
}

pub struct Node {
    pub name: String,
    pub ntype: NodeType,
    pub fs_data: usize,
}

pub struct DEntry {
    pub name: String,
    pub ntype: NodeType,
}

pub trait FileSystem {
    fn read_dir(&self, node: &Node, pos: usize) -> Result<Option<DEntry>, String>;
    fn create<'a>(&'a mut self, node: &Node, dent: &DEntry) -> Result<&'a Node, String>;
    fn read(&self, node: &Node, off: usize, data: &mut [u8]) -> Result<usize, String>;
    fn write(&mut self, node: &Node, off: usize, data: &[u8]) -> Result<usize, String>;
}
