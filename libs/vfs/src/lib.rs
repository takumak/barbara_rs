extern crate alloc;

use alloc::{
    boxed::Box,
    string::String,
    vec::Vec,
};

mod fscore;
mod fs_ramfs;

use fscore::FileSystem;
use fs_ramfs::RamFs;

struct Mount {
    filesystem: Box<dyn FileSystem>,
    mountpoint: Vec<String>,
}

struct Vfs {
    mount: Vec<Mount>,
}

impl Vfs {
    const fn new() -> Self {
        Self { mount: Vec::new() }
    }

    fn init(&mut self) {
        self.mount.insert(0, Mount {
            filesystem: Box::new(RamFs::new()),
            mountpoint: Vec::new(),
        });
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

    fn find_mount<'a, 'b>(&'a self, path: &'b str) -> Result<(&'a Mount, Vec<&'b str>), String> {
        let path: Vec<&'b str> = match Self::parse_path(path) {
            Ok(r) => r,
            Err(m) => return Err(m),
        };
        let mut mount: Option<&'a Mount> = None;
        for m in self.mount.iter().rev() {
            if path.iter().map(|s| *s).eq(m.mountpoint.iter().map(|s| s.as_str())) {
                mount = Some(m);
                break;
            }
        }
        assert!(mount.is_some());
        Ok((mount.unwrap(), Vec::from(&path[mount.unwrap().mountpoint.len()..])))
    }

    // fn read_dir(&self, path: &str) -> Result<DirIterator, String> {
    //     let (mount, mpath) = match self.find_mount(path) {
    //         Ok(r) => r,
    //         Err(m) => return Err(m),
    //     };
    //     mount.filesystem.read_dir(mpath.as_slice())
    // }

    // fn create_dir(&self, parent: &str, name: &str) -> Result<(), String> {
    //     let (mount, mpath) = match self.find_mount(parent) {
    //         Ok(r) => r,
    //         Err(m) => return Err(m),
    //     };
    //     mount.filesystem.create_dir(mpath.as_slice(), name)
    // }
}

static mut VFS: Vfs = Vfs::new();

pub unsafe fn init() {
    VFS.init();
}

// pub unsafe fn read_dir(path: &str) -> Result<DirIterator, String> {
//     VFS.read_dir(path)
// }

// pub unsafe fn create_dir(parent: &str, name: &str) -> Result<(), String> {
//     VFS.create_dir(parent, name)
// }

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
