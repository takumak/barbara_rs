use crate::fscore::{
    DEntry,
    NodeId,
    NODE_ID_ROOT,
    NodeType,
};

use alloc::collections::btree_map::BTreeMap;
use core::{
    cmp::{max, min},
    iter,
};

struct RamFsNode {
    id: NodeId,
    name: String,
    ntype: NodeType,
    children: Vec<NodeId>,
    file_body: Vec<u8>,
}

pub struct RamFs {
    fsnodes: BTreeMap<NodeId, RamFsNode>,
    next_node_id: NodeId,
}

impl RamFs {
    pub fn new() -> Self {
        let mut fsnodes: BTreeMap<NodeId, RamFsNode> = BTreeMap::new();
        fsnodes.insert(
            NODE_ID_ROOT,
            RamFsNode {
                id: NODE_ID_ROOT,
                name: String::from(""),
                ntype: NodeType::Directory,
                children: Vec::new(),
                file_body: Vec::new(),
            }
        );
        Self {
            fsnodes,
            next_node_id: NODE_ID_ROOT + 1,
        }
    }
}

impl crate::FileSystem for RamFs {
    fn read_dir(&self, dir: NodeId, pos: usize) -> Result<Option<DEntry>, String> {
        let dir_node = match self.fsnodes.get(&dir) {
            Some(n) => n,
            None => return Err(format!("Node not found (maybe a bug): id={}", dir)),
        };

        if dir_node.ntype != NodeType::Directory {
            return Err(format!("Attempt to read_dir() for a file: id={}", dir_node.id));
        }

        if pos >= dir_node.children.len() {
            Ok(None)
        } else {
            let fsnode = match self.fsnodes.get(&dir_node.children[pos]) {
                Some(n) => n,
                None => return Err(format!("Node not found (maybe a bug): id={}", dir_node.id)),
            };
            Ok(Some(DEntry {
                name: dir_node.name.to_owned(),
                ntype: node.ntype,
            }))
        }
    }

    fn create(&mut self, dir: NodeId, dent: &DEntry) -> Result<NodeId, String> {
        let dir_node = match self.fsnodes.get(&dir) {
            Some(n) => n,
            None => return Err(format!("Node not found (maybe a bug): id={}", dir)),
        };

        if dir_node.ntype != NodeType::Directory {
            return Err(format!("Attempt to create child node for a file: id={}", dir_node.id));
        }

        let node_id = self.next_node_id;
        self.next_node_id += 1;

        let newnode = RamFsNode {
            id: node_id,
            name: dent.name.to_owned(),
            ntype: dent.ntype,
            children: Vec::new(),
            file_body: Vec::new(),
        };

        self.fsnodes.insert(node_id, newnode);
        dir_node.children.push(node_id);

        Ok(node_id)
    }

    fn read(&self, file: NodeId, off: usize, data: &mut [u8]) -> Result<usize, String> {
        let file_node = match self.fsnodes.get(&file) {
            Some(n) => n,
            None => return Err(format!("Node not found (maybe a bug): id={}", file)),
        };

        if file_node.ntype != NodeType::RegularFile {
            return Err(format!("Attempt to read() for a directory: id={}", file_node.id));
        }

        if off >= file_node.file_body.len() {
            Ok(0)
        } else {
            let read_size = min(off + data.len(), file_node.file_body.len()) - off;
            data[..read_size].copy_from_slice(&file_node.file_body[off..read_size]);
            Ok(read_size)
        }
    }

    fn write(&mut self, file: NodeId, off: usize, data: &[u8]) -> Result<usize, String> {
        let file_node = match self.fsnodes.get_mut(&file) {
            Some(n) => n,
            None => return Err(format!("Node not found (maybe a bug): id={}", file)),
        };

        if file_node.ntype != NodeType::RegularFile {
            return Err(format!("Attempt to write() for a directory: id={}", file_node.id));
        }

        if off < file_node.file_body.len() {
            let overwrite_size = min(off + data.len(), file_node.file_body.len()) - off;
            file_node.file_body[off..off+overwrite_size].copy_from_slice(&data[..overwrite_size]);
        }

        if off > file_node.file_body.len() {
            file_node.file_body.extend(iter::repeat(0).take(off - file_node.file_body.len()));
        }

        if off + data.len() > file_node.file_body.len() {
            let from_i = max(file_node.file_body.len(), off) - off;
            file_node.file_body.extend_from_slice(&data[from_i..]);
        }

        Ok(data.len())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
    }
}
