#![feature(const_btree_new)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]

extern crate alloc;

use alloc::{
    boxed::Box,
    collections::btree_map::BTreeMap,
    string::String,
    vec::Vec,
};
use bitfield::bitfield;

bitfield!{
    pub OpenMode: u32 {
        READ[0];
        WRITE[1];
        CREATE[2];
        APPEND[3];
        TRUNC[4];
    }
}

mod fscore;
mod fs_ramfs;

use fscore::{
    DEntry,
    FileSystem,
    NodeId,
    NodeType,
    NODE_ID_ROOT,
};
use fs_ramfs::RamFs;

type MountId = usize;
type FileDescriptor = i32;

struct OpenedFile {
    mount_id: MountId,
    node_id: NodeId,
    mode: OpenMode,
    pos: usize,
}

struct Mount {
    id: MountId,
    mountpoint: Vec<String>,
    filesystem: Box<dyn FileSystem>,
}

struct Vfs {
    mount: Vec<Mount>,
    next_mnt_id: MountId,
    opened_files: BTreeMap<FileDescriptor, OpenedFile>,
    next_fd: FileDescriptor,
}

impl Vfs {
    const fn new() -> Self {
        Self {
            mount: Vec::new(),
            next_mnt_id: 1,
            opened_files: BTreeMap::new(),
            next_fd: 4,
        }
    }

    fn parse_path<'a>(path: &'a str) -> Result<Vec<&'a str>, String> {
        let mut path_vec: Vec<&'a str> = Vec::new();
        let mut skip: usize = 0;

        for name in path.split('/').rev() {
            if name.is_empty() || name == "." {
                // do nothing
            } else if name == ".." {
                skip += 1;
            } else if skip != 0 {
                skip -= 1;
            } else {
                path_vec.insert(0, name);
            }
        }

        Ok(path_vec)
    }

    fn mount(&mut self, mountpoint: &str, filesystem: Box<dyn FileSystem>) -> Result<(), String> {
        let mountpoint = match Self::parse_path(mountpoint) {
            Ok(v) => v.iter().map(|&s| String::from(s)).collect(),
            Err(m) => return Err(m)
        };

        self.mount.insert(0, Mount {
            id: self.next_mnt_id,
            mountpoint,
            filesystem,
        });

        self.next_mnt_id += 1;

        Ok(())
    }

    fn init(&mut self) {
        if let Err(m) = self.mount("/", Box::new(RamFs::new())) {
            panic!("Failed to mount root: {}", m)
        }
    }

    fn find_mount_by_path_mut<'a, 'b>(&'a mut self, path: &'b str) ->
        Result<(&'a mut Mount, Vec<&'b str>), String>
    {
        let path: Vec<&'b str> = match Self::parse_path(path) {
            Ok(r) => r,
            Err(m) => return Err(m),
        };
        let mut mount: Option<&'a mut Mount> = None;
        for m in self.mount.iter_mut() {
            if path.iter().take(m.mountpoint.len()).map(|s| *s).eq(
                m.mountpoint.iter().map(|s| s.as_str())) {
                mount = Some(m);
                break;
            }
        }
        assert!(mount.is_some());
        let mount = mount.unwrap();
        let mpath = Vec::from(&path[mount.mountpoint.len()..]);
        Ok((mount, mpath))
    }

    fn get_file_mount_from_fd(&mut self, fd: FileDescriptor) ->
        Result<(&mut OpenedFile, &mut Mount), String>
    {
        let file =
            match self.opened_files.get_mut(&fd) {
                Some(f) => f,
                None => return Err(format!("Invalid file descriptor: {}", fd)),
            };

        let mount =
            match self.mount.iter_mut().find(|m| m.id == file.mount_id) {
                Some(m) => m,
                None => return Err(format!("Invalid mount id: {}", file.mount_id)),
            };

        Ok((file, mount))
    }

    fn open(&mut self, path: &str, mode: OpenMode) -> Result<FileDescriptor, String> {
        let (mount, mpath) = match self.find_mount_by_path_mut(path) {
            Ok(r) => r,
            Err(m) => return Err(m),
        };

        /*

        TODO:
         * Support for OpenMode::APPEND
         * Support for OpenMode::TRUNC

         */

        let node_id =
            if mpath.len() == 0 {
                NODE_ID_ROOT
            } else {
                let dirname = &mpath[..(mpath.len() - 1)];
                let filename = mpath[mpath.len() - 1];

                let dir = match mount.filesystem.lookup_path(dirname) {
                    Ok(Some(node_id)) => node_id,
                    Ok(None) => return Err(format!("File not found: {:?}", path)),
                    Err(m) => return Err(m),
                };

                match mount.filesystem.lookup(dir, filename) {
                    Ok(Some(node_id)) => node_id,
                    Err(m) => return Err(m),
                    Ok(None) => {
                        if mode.all(OpenMode::CREATE) {
                            match mount.filesystem.create(
                                dir,
                                &DEntry {
                                    name: String::from(filename),
                                    ntype: NodeType::RegularFile,
                                })
                            {
                                Ok(node_id) => node_id,
                                Err(m) => return Err(m),
                            }
                        } else {
                            return Err(format!("File not found: {:?}", path))
                        }
                    },
                }
            };

        let mount_id = mount.id; // mount, *self borrow ends here

        let fd = self.next_fd;
        self.next_fd += 1;

        let file = OpenedFile {
            mount_id,
            node_id,
            mode,
            pos: 0,
        };

        self.opened_files.insert(fd, file);

        Ok(fd)
    }

    fn read(&mut self, fd: FileDescriptor, data: &mut [u8]) -> Result<usize, String> {
        let (file, mount) =
            match self.get_file_mount_from_fd(fd) {
                Ok((file, mount)) => (file, mount),
                Err(m) => return Err(m),
            };

        if !file.mode.all(OpenMode::READ) {
            return Err(format!("Permission error: fd={}", fd));
        }

        match mount.filesystem.read(file.node_id, file.pos, data) {
            Ok(size) => {
                file.pos += size;
                Ok(size)
            },
            Err(m) => Err(m),
        }
    }

    fn write(&mut self, fd: FileDescriptor, data: &[u8]) -> Result<usize, String> {
        let (file, mount) =
            match self.get_file_mount_from_fd(fd) {
                Ok((file, mount)) => (file, mount),
                Err(m) => return Err(m),
            };

        if !file.mode.all(OpenMode::WRITE) {
            return Err(format!("Permission error: fd={}", fd));
        }

        match mount.filesystem.write(file.node_id, file.pos, data) {
            Ok(size) => {
                file.pos += size;
                Ok(size)
            },
            Err(m) => Err(m),
        }
    }

    fn close(&mut self, fd: FileDescriptor) -> Result<(), String> {
        match self.opened_files.remove(&fd) {
            Some(f) => f,
            None => return Err(format!("Invalid file descriptor: {}", fd)),
        };
        Ok(())
    }

    fn mkdir(&mut self, path: &str) -> Result<(), String> {
        let (mount, mpath) = match self.find_mount_by_path_mut(path) {
            Ok(r) => r,
            Err(m) => return Err(m),
        };

        if mpath.len() == 0 {
            return Err(format!("Directory exists: {}", path));
        }

        let parent_name = &mpath[..(mpath.len() - 1)];
        let create_name = mpath[mpath.len() - 1];

        let dir = match mount.filesystem.lookup_path(parent_name) {
            Ok(Some(node_id)) => node_id,
            Ok(None) => return Err(format!("Directory not found: {:?}", path)),
            Err(m) => return Err(m),
        };

        match mount.filesystem.lookup(dir, create_name) {
            Ok(Some(_)) => Err(format!("Path exists: {}", path)),
            Err(m) => Err(m),
            Ok(None) => {
                match mount.filesystem.create(
                    dir,
                    &DEntry {
                        name: String::from(create_name),
                        ntype: NodeType::Directory,
                    })
                {
                    Ok(_) => Ok(()),
                    Err(m) => Err(m),
                }
            }
        }
    }
}

static mut VFS: Vfs = Vfs::new();

pub unsafe fn init() {
    VFS.init();
}

pub unsafe fn open(path: &str, mode: OpenMode) -> Result<FileDescriptor, String> {
    VFS.open(path, mode)
}

pub unsafe fn read(fd: FileDescriptor, data: &mut [u8]) -> Result<usize, String> {
    VFS.read(fd, data)
}

pub unsafe fn write(fd: FileDescriptor, data: &[u8]) -> Result<usize, String> {
    VFS.write(fd, data)
}

pub unsafe fn close(fd: FileDescriptor) -> Result<(), String> {
    VFS.close(fd)
}

pub unsafe fn mkdir(path: &str) -> Result<(), String> {
    VFS.mkdir(path)
}

#[cfg(test)]
mod tests {
    use crate::Vfs;

    #[test]
    fn parse_path() {
        assert_eq!(Vfs::parse_path(""), Ok(vec![]));
        assert_eq!(Vfs::parse_path("foo"), Ok(vec!["foo"]));
        assert_eq!(Vfs::parse_path(".."), Ok(vec![]));
        assert_eq!(Vfs::parse_path("/"), Ok(vec![]));
        assert_eq!(Vfs::parse_path("/foo"), Ok(vec!["foo"]));
        assert_eq!(Vfs::parse_path("/foo/bar"), Ok(vec!["foo", "bar"]));
        assert_eq!(Vfs::parse_path("/foo/bar/baz"), Ok(vec!["foo", "bar", "baz"]));
        assert_eq!(Vfs::parse_path("/foo///bar/."), Ok(vec!["foo", "bar"]));
        assert_eq!(Vfs::parse_path("/foo/../bar"), Ok(vec!["bar"]));
        assert_eq!(Vfs::parse_path("/foo/bar/.."), Ok(vec!["foo"]));
        assert_eq!(Vfs::parse_path("/foo/././//../bar"), Ok(vec!["bar"]));
        assert_eq!(Vfs::parse_path("/foo/.."), Ok(vec![]));
        assert_eq!(Vfs::parse_path("/foo/../.."), Ok(vec![]));
        assert_eq!(Vfs::parse_path("/.."), Ok(vec![]));
    }
}
