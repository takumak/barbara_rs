type NodeId = usize;
const NODE_ID_ROOT: NodeId = 0;

use crate::fscore::{
    DEntry,
    Node,
    NodeType,
};

use alloc::collections::btree_map::BTreeMap;
use core::{
    cmp::{max, min},
    iter,
};

struct RamFsNode {
    node: Node,
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
                node: Node {
                    name: String::from(""),
                    ntype: NodeType::Directory,
                    fs_data: NODE_ID_ROOT,
                },
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
    fn read_dir(&self, node: &Node, pos: usize) -> Result<Option<DEntry>, String> {
        let parent = match self.fsnodes.get(&node.fs_data) {
            Some(n) => n,
            None => return Err(format!("Node not found (maybe a bug): id={}", node.fs_data)),
        };

        if parent.node.ntype != NodeType::Directory {
            return Err(format!("Attempt to read_dir() for a file: id={}", node.fs_data));
        }

        if pos >= parent.children.len() {
            Ok(None)
        } else {
            let fsnode = match self.fsnodes.get(&parent.children[pos]) {
                Some(n) => n,
                None => return Err(format!("Node not found (maybe a bug): id={}", node.fs_data)),
            };
            Ok(Some(DEntry {
                name: fsnode.node.name.to_owned(),
                ntype: fsnode.node.ntype,
            }))
        }
    }

    fn create<'a>(&'a mut self, node: &Node, dent: &DEntry) -> Result<&'a Node, String> {
        let parent = match self.fsnodes.get(&node.fs_data) {
            Some(n) => n,
            None => return Err(format!("Node not found (maybe a bug): id={}", node.fs_data)),
        };

        if parent.node.ntype != NodeType::Directory {
            return Err(format!("Attempt to create child node for a file: id={}", node.fs_data));
        }

        let node_id = self.next_node_id;
        self.next_node_id += 1;

        let fsnode = RamFsNode {
            node: Node {
                name: dent.name.to_owned(),
                ntype: dent.ntype,
                fs_data: node_id,
            },
            children: Vec::new(),
            file_body: Vec::new(),
        };

        self.fsnodes.insert(node_id, fsnode);
        Ok(&self.fsnodes.get(&node_id).unwrap().node)
    }

    fn read(&self, node: &Node, off: usize, data: &mut [u8]) -> Result<usize, String> {
        let file = match self.fsnodes.get(&node.fs_data) {
            Some(n) => n,
            None => return Err(format!("Node not found (maybe a bug): id={}", node.fs_data)),
        };

        if file.node.ntype != NodeType::RegularFile {
            return Err(format!("Attempt to read() for a directory: id={}", node.fs_data));
        }

        if off >= file.file_body.len() {
            Ok(0)
        } else {
            let read_size = min(off + data.len(), file.file_body.len()) - off;
            data[..read_size].copy_from_slice(&file.file_body[off..read_size]);
            Ok(read_size)
        }
    }

    fn write(&mut self, node: &Node, off: usize, data: &[u8]) -> Result<usize, String> {
        let file = match self.fsnodes.get_mut(&node.fs_data) {
            Some(n) => n,
            None => return Err(format!("Node not found (maybe a bug): id={}", node.fs_data)),
        };

        if file.node.ntype != NodeType::RegularFile {
            return Err(format!("Attempt to write() for a directory: id={}", node.fs_data));
        }

        if off < file.file_body.len() {
            let overwrite_size = min(off + data.len(), file.file_body.len()) - off;
            file.file_body[off..off+overwrite_size].copy_from_slice(&data[..overwrite_size]);
        }

        if off > file.file_body.len() {
            file.file_body.extend(iter::repeat(0).take(off - file.file_body.len()));
        }

        if off + data.len() > file.file_body.len() {
            let from_i = max(file.file_body.len(), off) - off;
            file.file_body.extend_from_slice(&data[from_i..]);
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
