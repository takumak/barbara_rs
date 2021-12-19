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
    fn readdir(&self, dir: NodeId, pos: usize) -> Result<Option<(DEntry, NodeId)>, String> {
        let dir_node = self.fsnodes.get(&dir).unwrap();

        if dir_node.ntype != NodeType::Directory {
            return Err(format!("Attempt to readdir() for a file: id={}", dir_node.id));
        }

        if pos >= dir_node.children.len() {
            Ok(None)
        } else {
            let fsnode = self.fsnodes.get(&dir_node.children[pos]).unwrap();
            Ok(Some((
                DEntry {
                    name: fsnode.name.to_owned(),
                    ntype: fsnode.ntype,
                },
                fsnode.id,
            )))
        }
    }

    fn create(&mut self, dir: NodeId, dent: &DEntry) -> Result<NodeId, String> {
        let dir_node = self.fsnodes.get_mut(&dir).unwrap();

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

        dir_node.children.push(node_id);
        self.fsnodes.insert(node_id, newnode);

        Ok(node_id)
    }

    fn read(&self, file: NodeId, off: usize, data: &mut [u8]) -> Result<usize, String> {
        let file_node = self.fsnodes.get(&file).unwrap();

        if file_node.ntype != NodeType::RegularFile {
            return Err(format!("Attempt to read() for a directory: id={}", file_node.id));
        }

        if off >= file_node.file_body.len() {
            Ok(0)
        } else {
            let read_size = min(off + data.len(), file_node.file_body.len()) - off;
            data[..read_size].copy_from_slice(&file_node.file_body[off..(off+read_size)]);
            Ok(read_size)
        }
    }

    fn write(&mut self, file: NodeId, off: usize, data: &[u8]) -> Result<usize, String> {
        let file_node = self.fsnodes.get_mut(&file).unwrap();

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

    fn truncate(&mut self, file: NodeId, len: usize) -> Result<(), String> {
        let file_node = self.fsnodes.get_mut(&file).unwrap();

        if file_node.ntype != NodeType::RegularFile {
            return Err(format!("Attempt to truncate() for a directory: id={}", file_node.id));
        }

        file_node.file_body.truncate(len);
        Ok(())
    }

    fn getsize(&self, file: NodeId) -> Result<usize, String> {
        let file_node = self.fsnodes.get(&file).unwrap();
        Ok(file_node.file_body.len())
    }
}


#[cfg(test)]
mod tests {
    use crate::{
        DEntry,
        FileSystem,
        NodeType,
        NODE_ID_ROOT,
        RamFs,
    };

    #[test]
    fn create_on_regular_file() {
        let mut ramfs = RamFs::new();

        let parent = ramfs.create(
            NODE_ID_ROOT,
            &DEntry {
                name: String::from("foo"),
                ntype: NodeType::RegularFile,
            }
        ).unwrap();

        ramfs.create(
            parent,
            &DEntry {
                name: String::from("bar"),
                ntype: NodeType::RegularFile,
            }
        ).expect_err("create on a regular file unexpectedly succeed");
    }

    #[test]
    fn read_directory() {
        let mut ramfs = RamFs::new();

        let node_id = ramfs.create(
            NODE_ID_ROOT,
            &DEntry {
                name: String::from("foo"),
                ntype: NodeType::Directory,
            }
        ).unwrap();

        let mut buf: [u8; 10] = [0; 10];
        ramfs.read(node_id, 0, &mut buf)
            .expect_err("read on a directory unexpectedly succeed");
    }

    #[test]
    fn write_directory() {
        let mut ramfs = RamFs::new();

        let node_id = ramfs.create(
            NODE_ID_ROOT,
            &DEntry {
                name: String::from("foo"),
                ntype: NodeType::Directory,
            }
        ).unwrap();

        let buf: [u8; 10] = [0; 10];
        ramfs.write(node_id, 0, &buf)
            .expect_err("write on a directory unexpectedly succeed");
    }

    #[test]
    fn write_sparse() {
        let mut ramfs = RamFs::new();

        let node_id = ramfs.create(
            NODE_ID_ROOT,
            &DEntry {
                name: String::from("foo"),
                ntype: NodeType::RegularFile,
            }
        ).unwrap();

        let buf1: [u8; 2] = [1; 2];
        let buf2: [u8; 2] = [2; 2];
        ramfs.write(node_id, 0, &buf1).unwrap();
        ramfs.write(node_id, 4, &buf2).unwrap();

        let mut buf: [u8; 6] = [1; 6];
        ramfs.read(node_id, 0, &mut buf).unwrap();
        assert_eq!(buf, [1u8, 1u8, 0u8, 0u8, 2u8, 2u8]);
    }
}
