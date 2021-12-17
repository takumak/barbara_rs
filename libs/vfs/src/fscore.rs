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
    fn read_dir(&self, dir: NodeId, pos: usize) -> Result<Option<(DEntry, NodeId)>, String>;
    fn create(&mut self, dir: NodeId, dent: &DEntry) -> Result<NodeId, String>;
    fn read(&self, file: NodeId, off: usize, data: &mut [u8]) -> Result<usize, String>;
    fn write(&mut self, file: NodeId, off: usize, data: &[u8]) -> Result<usize, String>;

    fn lookup(&self, dir: NodeId, name: &str) -> Result<Option<NodeId>, String> {
        let mut pos: usize = 0;
        loop {
            match self.read_dir(dir, pos) {
                Ok(Some((ent, node_id))) => {
                    if ent.name == *name {
                        return Ok(Some(node_id))
                    }
                },
                Ok(None) => return Ok(None),
                Err(m) => return Err(m),
            }
            pos += 1
        };
    }

    fn lookup_path(&self, path: &[&str]) -> Result<Option<NodeId>, String> {
        let mut dir;
        let mut file = NODE_ID_ROOT;
        for name in path {
            dir = file;
            file = match self.lookup(dir, name) {
                Ok(Some(node_id)) => node_id,
                Ok(None) => return Ok(None),
                Err(m) => return Err(m),
            };
        }
        return Ok(Some(file));
    }
}
