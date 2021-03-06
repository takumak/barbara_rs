#![feature(const_btree_new)]
#![feature(no_coverage)]

extern crate alloc;
extern crate posix;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, string::String, vec::Vec};
use bitfield::bitfield;

bitfield! {
    pub OpenMode: u32 {
        READ[0];
        WRITE[1];
        CREATE[2];
        APPEND[3];
        TRUNC[4];
    }
}

mod fs_ramfs;
mod fscore;

use fs_ramfs::RamFs;
use fscore::{DEntry, FileSystem, FsError, NodeId, NodeType, NODE_ID_ROOT};

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

    fn parse_path<'a>(path: &'a str) -> Vec<&'a str> {
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

        path_vec
    }

    fn mount(&mut self, mountpoint: &str, filesystem: Box<dyn FileSystem>) -> Result<(), FsError> {
        if self.mount.is_empty() {
            if mountpoint != "/" {
                return Err(FsError::new(
                    posix::Errno::EINVAL,
                    format!(
                        "Non-root mountpoint specified for the first time: {}",
                        mountpoint
                    ),
                ));
            }
        } else {
            let fd = self.open(mountpoint, OpenMode::READ)?;
            let dent = self.readdir(fd);
            self.close(fd).expect("Failed to close file descriptor");
            if dent?.is_some() {
                return Err(FsError::new(
                    posix::Errno::EEXIST,
                    format!("Mountpoint is not empty: {}", mountpoint),
                ));
            }
        }

        let mountpoint = Self::parse_path(mountpoint)
            .iter()
            .map(|&s| String::from(s))
            .collect();

        self.mount.insert(
            0,
            Mount {
                id: self.next_mnt_id,
                mountpoint,
                filesystem,
            },
        );

        self.next_mnt_id += 1;

        Ok(())
    }

    fn init(&mut self) {
        self.mount("/", Box::new(RamFs::new()))
            .expect("Failed to mount root");
    }

    fn find_mount_by_path_mut<'a, 'b>(
        &'a mut self,
        path: &'b str,
    ) -> (&'a mut Mount, Vec<&'b str>) {
        let path: Vec<&'b str> = Self::parse_path(path);
        let mut mount: Option<&'a mut Mount> = None;
        for m in self.mount.iter_mut() {
            if path
                .iter()
                .take(m.mountpoint.len()).copied()
                .eq(m.mountpoint.iter().map(|s| s.as_str()))
            {
                mount = Some(m);
                break;
            }
        }
        assert!(mount.is_some());
        let mount = mount.unwrap();
        let mpath = Vec::from(&path[mount.mountpoint.len()..]);
        (mount, mpath)
    }

    fn get_file_mount_from_fd(
        &mut self,
        fd: FileDescriptor,
    ) -> Result<(&mut OpenedFile, &mut Mount), FsError> {
        let file = self.opened_files.get_mut(&fd).ok_or(FsError::new(
            posix::Errno::EBADF,
            format!("Invalid file descriptor: {}", fd),
        ))?;

        let mount = self
            .mount
            .iter_mut()
            .find(|m| m.id == file.mount_id)
            .unwrap();

        Ok((file, mount))
    }

    fn open(&mut self, path: &str, mode: OpenMode) -> Result<FileDescriptor, FsError> {
        let (mount, mpath) = self.find_mount_by_path_mut(path);

        let node_id = if mpath.is_empty() {
            NODE_ID_ROOT
        } else {
            let dirname = &mpath[..(mpath.len() - 1)];
            let filename = mpath[mpath.len() - 1];

            let dir = mount.filesystem.lookup_path(dirname)?.ok_or(FsError::new(
                posix::Errno::ENOENT,
                format!("File not found: {:?}", path),
            ))?;

            match mount.filesystem.lookup(dir, filename)? {
                Some(node_id) => node_id,
                None => {
                    if mode.is_set(OpenMode::CREATE) {
                        mount.filesystem.create(
                            dir,
                            &DEntry {
                                name: String::from(filename),
                                ntype: NodeType::RegularFile,
                            },
                        )?
                    } else {
                        return Err(FsError::new(
                            posix::Errno::ENOENT,
                            format!("File not found: {:?}", path),
                        ));
                    }
                }
            }
        };

        if mode.is_set(OpenMode::TRUNC) {
            mount.filesystem.truncate(node_id, 0)?;
        }

        let pos: usize = if mode.is_set(OpenMode::APPEND) {
            mount.filesystem.getsize(node_id)?
        } else {
            0
        };

        let mount_id = mount.id; // mount, *self borrow ends here

        let fd = self.next_fd;
        self.next_fd += 1;

        let file = OpenedFile {
            mount_id,
            node_id,
            mode,
            pos,
        };

        self.opened_files.insert(fd, file);

        Ok(fd)
    }

    fn read(&mut self, fd: FileDescriptor, data: &mut [u8]) -> Result<usize, FsError> {
        let (file, mount) = self.get_file_mount_from_fd(fd)?;

        if !file.mode.is_set(OpenMode::READ) {
            return Err(FsError::new(
                posix::Errno::EPERM,
                format!("Permission error: fd={}", fd),
            ));
        }

        let size = mount.filesystem.read(file.node_id, file.pos, data)?;
        file.pos += size;
        Ok(size)
    }

    fn write(&mut self, fd: FileDescriptor, data: &[u8]) -> Result<usize, FsError> {
        let (file, mount) = self.get_file_mount_from_fd(fd)?;

        if !file.mode.is_set(OpenMode::WRITE) {
            return Err(FsError::new(
                posix::Errno::EPERM,
                format!("Permission error: fd={}", fd),
            ));
        }

        let size = mount.filesystem.write(file.node_id, file.pos, data)?;
        file.pos += size;
        Ok(size)
    }

    fn close(&mut self, fd: FileDescriptor) -> Result<(), FsError> {
        self.opened_files.remove(&fd).ok_or(FsError::new(
            posix::Errno::EBADF,
            format!("Invalid file descriptor: {}", fd),
        ))?;
        Ok(())
    }

    fn mkdir(&mut self, path: &str) -> Result<(), FsError> {
        let (mount, mpath) = self.find_mount_by_path_mut(path);

        if mpath.is_empty() {
            return Err(FsError::new(
                posix::Errno::EEXIST,
                format!("Directory exists: {}", path),
            ));
        }

        let parent_name = &mpath[..(mpath.len() - 1)];
        let create_name = mpath[mpath.len() - 1];

        let dir = mount
            .filesystem
            .lookup_path(parent_name)?
            .ok_or(FsError::new(
                posix::Errno::ENOENT,
                format!("Directory not found: {:?}", path),
            ))?;

        match mount.filesystem.lookup(dir, create_name)? {
            Some(_) => Err(FsError::new(
                posix::Errno::EEXIST,
                format!("Path exists: {}", path),
            )),
            None => {
                mount.filesystem.create(
                    dir,
                    &DEntry {
                        name: String::from(create_name),
                        ntype: NodeType::Directory,
                    },
                )?;
                Ok(())
            }
        }
    }

    fn readdir(&mut self, fd: FileDescriptor) -> Result<Option<DEntry>, FsError> {
        let (file, mount) = self.get_file_mount_from_fd(fd)?;

        let res = match mount.filesystem.readdir(file.node_id, file.pos)? {
            Some((dent, _)) => {
                file.pos += 1;
                Some(dent)
            }
            None => None,
        };
        Ok(res)
    }
}

static mut VFS: Vfs = Vfs::new();

#[no_coverage]
pub unsafe fn init() {
    VFS.init();
}

#[no_coverage]
pub unsafe fn open(path: &str, mode: OpenMode) -> Result<FileDescriptor, FsError> {
    VFS.open(path, mode)
}

#[no_coverage]
pub unsafe fn read(fd: FileDescriptor, data: &mut [u8]) -> Result<usize, FsError> {
    VFS.read(fd, data)
}

#[no_coverage]
pub unsafe fn write(fd: FileDescriptor, data: &[u8]) -> Result<usize, FsError> {
    VFS.write(fd, data)
}

#[no_coverage]
pub unsafe fn close(fd: FileDescriptor) -> Result<(), FsError> {
    VFS.close(fd)
}

#[no_coverage]
pub unsafe fn mkdir(path: &str) -> Result<(), FsError> {
    VFS.mkdir(path)
}

#[no_coverage]
pub unsafe fn readdir(fd: FileDescriptor) -> Result<Option<DEntry>, FsError> {
    VFS.readdir(fd)
}

#[cfg(test)]
mod tests {
    use crate::{NodeType, OpenMode, RamFs, Vfs};

    #[test]
    fn parse_path() {
        assert_eq!(Vfs::parse_path(""), Vec::<String>::new());
        assert_eq!(Vfs::parse_path("foo"), vec!["foo"]);
        assert_eq!(Vfs::parse_path(".."), Vec::<String>::new());
        assert_eq!(Vfs::parse_path("/"), Vec::<String>::new());
        assert_eq!(Vfs::parse_path("/foo"), vec!["foo"]);
        assert_eq!(Vfs::parse_path("/foo/bar"), vec!["foo", "bar"]);
        assert_eq!(Vfs::parse_path("/foo/bar/baz"), vec!["foo", "bar", "baz"]);
        assert_eq!(Vfs::parse_path("/foo///bar/."), vec!["foo", "bar"]);
        assert_eq!(Vfs::parse_path("/foo/../bar"), vec!["bar"]);
        assert_eq!(Vfs::parse_path("/foo/bar/.."), vec!["foo"]);
        assert_eq!(Vfs::parse_path("/foo/././//../bar"), vec!["bar"]);
        assert_eq!(Vfs::parse_path("/foo/.."), Vec::<String>::new());
        assert_eq!(Vfs::parse_path("/foo/../.."), Vec::<String>::new());
        assert_eq!(Vfs::parse_path("/.."), Vec::<String>::new());
    }

    #[test]
    fn write_read() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "foo\n".as_bytes()).unwrap();
        vfs.write(fd, "bar\n".as_bytes()).unwrap();
        vfs.write(fd, "baz\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs.open("/foo.txt", OpenMode::READ).unwrap();
        let mut buf: [u8; 30] = [0; 30];
        let len = vfs.read(fd, &mut buf).unwrap();
        vfs.close(fd).unwrap();

        assert_eq!(buf[..len], *"foo\nbar\nbaz\n".as_bytes());
    }

    #[test]
    fn not_found() {
        let mut vfs = Vfs::new();
        vfs.init();

        vfs.open("/foo.txt", OpenMode::WRITE)
            .expect_err("open() for a non-exist file unexpectedly succeed");

        vfs.open("/foo.txt", OpenMode::READ)
            .expect_err("open() for a non-exist file unexpectedly succeed");
    }

    #[test]
    fn read_and_write_shoud_share_file_position() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "foo\n".as_bytes()).unwrap();
        vfs.write(fd, "bar\n".as_bytes()).unwrap();
        vfs.write(fd, "baz\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs
            .open("/foo.txt", OpenMode::READ | OpenMode::WRITE)
            .unwrap();
        let mut buf: [u8; 30] = [0; 30];
        let mut pos: usize = 0;
        pos += vfs.read(fd, &mut buf[pos..(pos + 4)]).unwrap();
        vfs.write(fd, "***\n".as_bytes()).unwrap();
        pos += vfs.read(fd, &mut buf[pos..(pos + 4)]).unwrap();
        vfs.close(fd).unwrap();

        assert_eq!(buf[..pos], *"foo\nbaz\n".as_bytes());

        let fd = vfs.open("/foo.txt", OpenMode::READ).unwrap();
        let mut buf: [u8; 30] = [0; 30];
        let len = vfs.read(fd, &mut buf).unwrap();
        vfs.close(fd).unwrap();

        assert_eq!(buf[..len], *"foo\n***\nbaz\n".as_bytes());
    }

    #[test]
    fn permission_read_file() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "foo\n".as_bytes()).unwrap();
        vfs.write(fd, "bar\n".as_bytes()).unwrap();
        vfs.write(fd, "baz\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs.open("/foo.txt", OpenMode::WRITE).unwrap();
        let mut buf: [u8; 30] = [0; 30];

        vfs.read(fd, &mut buf)
            .expect_err("OpenMode permission violating read() request unexpectedly succeed");
    }

    #[test]
    fn permission_write_file() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "foo\n".as_bytes()).unwrap();
        vfs.write(fd, "bar\n".as_bytes()).unwrap();
        vfs.write(fd, "baz\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs.open("/foo.txt", OpenMode::READ).unwrap();

        vfs.write(fd, "a".as_bytes())
            .expect_err("OpenMode permission violating write() request unexpectedly succeed");
    }

    #[test]
    fn write_overwrite() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "foo\n".as_bytes()).unwrap();
        vfs.write(fd, "bar\n".as_bytes()).unwrap();
        vfs.write(fd, "baz\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs.open("/foo.txt", OpenMode::WRITE).unwrap();
        vfs.write(fd, "***\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs.open("/foo.txt", OpenMode::READ).unwrap();
        let mut buf: [u8; 30] = [0; 30];
        let len = vfs.read(fd, &mut buf).unwrap();
        vfs.close(fd).unwrap();

        assert_eq!(buf[..len], *"***\nbar\nbaz\n".as_bytes());
    }

    #[test]
    fn trunc() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "foo\n".as_bytes()).unwrap();
        vfs.write(fd, "bar\n".as_bytes()).unwrap();
        vfs.write(fd, "baz\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::TRUNC)
            .unwrap();
        vfs.write(fd, "***\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs.open("/foo.txt", OpenMode::READ).unwrap();
        let mut buf: [u8; 30] = [0; 30];
        let len = vfs.read(fd, &mut buf).unwrap();
        vfs.close(fd).unwrap();

        assert_eq!(buf[..len], *"***\n".as_bytes());
    }

    #[test]
    fn prefer_trunc_than_append() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "foo\n".as_bytes()).unwrap();
        vfs.write(fd, "bar\n".as_bytes()).unwrap();
        vfs.write(fd, "baz\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs
            .open(
                "/foo.txt",
                OpenMode::WRITE | OpenMode::TRUNC | OpenMode::APPEND,
            )
            .unwrap();
        vfs.write(fd, "***\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs.open("/foo.txt", OpenMode::READ).unwrap();
        let mut buf: [u8; 30] = [0; 30];
        let len = vfs.read(fd, &mut buf).unwrap();
        vfs.close(fd).unwrap();

        assert_eq!(buf[..len], *"***\n".as_bytes());
    }

    #[test]
    fn append() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "foo\n".as_bytes()).unwrap();
        vfs.write(fd, "bar\n".as_bytes()).unwrap();
        vfs.write(fd, "baz\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::APPEND)
            .unwrap();
        vfs.write(fd, "***\n".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs.open("/foo.txt", OpenMode::READ).unwrap();
        let mut buf: [u8; 30] = [0; 30];
        let len = vfs.read(fd, &mut buf).unwrap();
        vfs.close(fd).unwrap();

        assert_eq!(buf[..len], *"foo\nbar\nbaz\n***\n".as_bytes());
    }

    #[test]
    fn mkdir() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "/foo.txt".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        vfs.mkdir("/hoge").unwrap();

        let fd = vfs
            .open("/hoge/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "/hoge/foo.txt".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        // validate

        let fd = vfs.open("/foo.txt", OpenMode::READ).unwrap();
        let mut buf: [u8; 30] = [0; 30];
        let len = vfs.read(fd, &mut buf).unwrap();
        vfs.close(fd).unwrap();
        assert_eq!(buf[..len], *"/foo.txt".as_bytes());

        let fd = vfs.open("/hoge/foo.txt", OpenMode::READ).unwrap();
        let mut buf: [u8; 30] = [0; 30];
        let len = vfs.read(fd, &mut buf).unwrap();
        vfs.close(fd).unwrap();
        assert_eq!(buf[..len], *"/hoge/foo.txt".as_bytes());
    }

    #[test]
    fn readdir() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs.open("/foo.txt", OpenMode::CREATE).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs.open("/bar.txt", OpenMode::CREATE).unwrap();
        vfs.close(fd).unwrap();

        // validate

        let fd = vfs.open("/", OpenMode::READ).unwrap();
        let mut files = vec![];
        while let Some(dent) = vfs.readdir(fd).unwrap() {
            files.push(dent.name);
        }
        files.sort_unstable();
        assert_eq!(files, ["bar.txt", "foo.txt"])
    }

    #[test]
    fn openmode_clone() {
        assert_eq!(OpenMode::CREATE, OpenMode::CREATE.clone());
    }

    #[test]
    fn openmode_debug() {
        assert_eq!(format!("{:?}", OpenMode::CREATE), "OpenMode(4)");
    }

    #[test]
    fn mount_nonroot_for_1st_time() {
        let mut vfs = Vfs::new();
        let r = vfs.mount("/foo", Box::new(RamFs::new()));
        r.expect_err("Non-root mount for the 1st time unexpectedly succeed");
    }

    #[test]
    fn mount_on_nonempty_dir() {
        let mut vfs = Vfs::new();
        vfs.init();

        vfs.mkdir("/foo").unwrap();
        vfs.mkdir("/foo/bar").unwrap();

        let r = vfs.mount("/foo", Box::new(RamFs::new()));
        r.expect_err("Mount for a non-empty directory unexpectedly succeed");
    }

    #[test]
    fn mountpoint_not_exist() {
        let mut vfs = Vfs::new();
        vfs.init();

        vfs.mount("/foo", Box::new(RamFs::new()))
            .expect_err("Mount on non-exist mountpoint unexpectedly succeed");
    }

    #[test]
    fn non_root_mount() {
        let mut vfs = Vfs::new();
        vfs.init();

        vfs.mkdir("/foo").unwrap();
        vfs.mount("/foo", Box::new(RamFs::new())).unwrap();
        vfs.mkdir("/foo/bar").unwrap();
        vfs.mkdir("/baz").unwrap();
    }

    #[test]
    fn mount_on_file() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs.open("/foo", OpenMode::CREATE).unwrap();
        vfs.close(fd).unwrap();

        vfs.mount("/foo", Box::new(RamFs::new()))
            .expect_err("Mount on a file unexpectedly succeed");
    }

    #[test]
    fn mkdir_already_exists() {
        let mut vfs = Vfs::new();
        vfs.init();

        vfs.mkdir("/foo").unwrap();

        vfs.mkdir("/foo")
            .expect_err("mkdir on already exist directory unexpectedly succeed");
    }

    #[test]
    fn mkdir_on_mountpoint() {
        let mut vfs = Vfs::new();
        vfs.init();

        vfs.mkdir("/foo").unwrap();
        vfs.mount("/foo", Box::new(RamFs::new())).unwrap();

        vfs.mkdir("/foo")
            .expect_err("mkdir on a mountpoint unexpectedly succeed");
    }

    #[test]
    fn mkdir_parent_not_found() {
        let mut vfs = Vfs::new();
        vfs.init();

        vfs.mkdir("/foo/bar")
            .expect_err("mkdir on non-existent parent unexpectedly succeed");
    }

    #[test]
    fn read_invalid_fd() {
        let mut vfs = Vfs::new();
        vfs.init();
        let mut buf: [u8; 10] = [0; 10];
        vfs.read(100, &mut buf)
            .expect_err("read on invalid fd unexpectedly succeed");
    }

    #[test]
    fn write_invalid_fd() {
        let mut vfs = Vfs::new();
        vfs.init();
        let buf: [u8; 10] = [0; 10];
        vfs.write(100, &buf)
            .expect_err("write on invalid fd unexpectedly succeed");
    }

    #[test]
    fn close_invalid_fd() {
        let mut vfs = Vfs::new();
        vfs.init();
        vfs.close(100)
            .expect_err("close on invalid fd unexpectedly succeed");
    }

    #[test]
    fn readdir_invalid_fd() {
        let mut vfs = Vfs::new();
        vfs.init();
        vfs.readdir(100)
            .expect_err("read on invalid fd unexpectedly succeed");
    }

    #[test]
    fn open_parent_not_found() {
        let mut vfs = Vfs::new();
        vfs.init();
        vfs.open("/foo/bar.txt", OpenMode::CREATE)
            .expect_err("open for non-existent path unexpectedly succeed");
    }

    #[test]
    fn nodetype_clone() {
        let t = NodeType::Directory;
        assert_eq!(t, t.clone());
    }

    #[test]
    fn nodetype_debug() {
        assert_eq!(format!("{:?}", NodeType::Directory), "Directory");
    }

    #[test]
    fn read_eof() {
        let mut vfs = Vfs::new();
        vfs.init();

        let fd = vfs
            .open("/foo.txt", OpenMode::WRITE | OpenMode::CREATE)
            .unwrap();
        vfs.write(fd, "foo".as_bytes()).unwrap();
        vfs.close(fd).unwrap();

        let fd = vfs.open("/foo.txt", OpenMode::READ).unwrap();
        let mut buf: [u8; 30] = [0; 30];
        assert_eq!(vfs.read(fd, &mut buf).unwrap(), 3);
        assert_eq!(vfs.read(fd, &mut buf).unwrap(), 0);
        vfs.close(fd).unwrap();
    }

    #[test]
    fn trunc_directory() {
        let mut vfs = Vfs::new();
        vfs.init();

        vfs.mkdir("/foo").unwrap();

        vfs.open("/foo", OpenMode::TRUNC)
            .expect_err("open with OpenMode::TRUNC on a directory unexpectedly succeed");
    }
}
