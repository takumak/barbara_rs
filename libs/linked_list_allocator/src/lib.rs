#![cfg_attr(not(test), no_std)]

use core::ptr;
use core::alloc::GlobalAlloc;
extern crate alloc;
use alloc::alloc::Layout;

const MIN_SIZE: usize = 32 << 10;

const fn align_up(alignment: usize, value: usize) -> usize {
    if value % alignment != 0 {
        value + alignment - (value % alignment)
    } else {
        value
    }
}

struct Area {
    addr: usize,
    size: usize,
    prev: *mut Area,
    next: *mut Area,
}

impl Area {
    fn take_next_mut(&mut self) -> Option<&mut Area> {
        if self.prev == self.next {
            return None
        }

        let item = unsafe { &mut *self.next };
        let next = unsafe { &mut *item.next };

        self.next = next;
        next.prev = self;

        item.prev = item;
        item.next = item;

        Some(item)
    }

    fn append(&mut self, area: &mut Area) {
        let prev = unsafe { &mut *self.prev };

        prev.next = area;
        area.prev = prev;
        area.next = self;
        self.prev = area;
    }

    fn right(&self) -> usize {
        self.addr + self.size
    }

    fn is_overwrapping(&self, area: &Area) -> Option<(usize, usize)> {
        use core::cmp::min;

        //  case 1
        //    area      <------>
        //    self  <------>
        if self.addr < area.addr && area.addr < self.right() {
            return Some((area.addr, min(area.right(), self.right()) - area.addr))
        }

        //  case 2
        //    area  <------>
        //    self      <------>
        if self.addr < area.right() && area.right() < self.right() {
            return Some((self.addr, min(area.right(), self.right()) - self.addr))
        }

        //  case 3
        //    area  <-------------->
        //    self      <------>
        if area.addr <= self.addr && self.right() <= area.right() {
            return Some((self.addr, self.size))
        }

        None
    }
}

struct AreaIterator {
    head: *mut Area,
    curr: *mut Area,
}

impl AreaIterator {
    fn new(head: *mut Area) -> Self {
        let curr = (unsafe { &*head }).next;
        Self { head, curr }
    }
}

impl Iterator for AreaIterator {
    type Item = &'static mut Area;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.head {
            return None;
        }

        let ret = self.curr;
        self.curr = (unsafe { &*self.curr }).next;
        Some(unsafe { &mut *ret })
    }
}

struct AreaList {
    head: *mut Area,
}

impl AreaList {
    const fn new() -> AreaList {
        AreaList { head: ptr::null_mut() }
    }

    fn init(&mut self, area_ary: &mut [Area]) {
        self.head = &mut area_ary[0];
        for idx in 0..area_ary.len() {
            let item = unsafe { &mut *self.head.add(idx) };
            let p = (idx + area_ary.len() - 1) % area_ary.len();
            let n = (idx + 1) % area_ary.len();
            item.addr = 0;
            item.size = 0;
            item.prev = unsafe { self.head.add(p) };
            item.next = unsafe { self.head.add(n) };
        }
    }

    fn iter_mut(&self) -> AreaIterator {
        AreaIterator::new(self.head)
    }

    fn take_mut(&self) -> Option<&mut Area> {
        unsafe { &mut *self.head }.take_next_mut()
    }

    fn append(&self, area: &mut Area) {
        unsafe { &mut *self.head }.append(area)
    }
}

pub struct LinkedListAllocator {
    initialized: bool,
    free: AreaList,
    unused: AreaList,
    alloc_top: usize,
    alloc_end: usize,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        LinkedListAllocator {
            initialized: false,
            free: AreaList::new(),
            unused: AreaList::new(),
            alloc_top: 0,
            alloc_end: 0,
        }
    }

    pub fn init(&mut self, mem_top: usize, mem_end: usize) {
        use core::mem::size_of;

        if mem_end - mem_top < MIN_SIZE {
            panic!("Heap area too small: given={:#08x}, required={:#08x}",
                   mem_end - mem_top, MIN_SIZE);
        }

        let alloc_top = mem_top +
            align_up(
                size_of::<Area>(),
                (mem_end - mem_top) / 10);

        let count = (alloc_top - mem_top) / (size_of::<Area>());
        let area_ary = unsafe {
            core::slice::from_raw_parts_mut(
                mem_top as *mut Area, count)
        };

        self.free.init(&mut area_ary[0..1]);
        self.unused.init(&mut area_ary[1..]);
        self.alloc_top = alloc_top;
        self.alloc_end = mem_end;

        self.initialized = true;

        unsafe {
            self.dealloc(
                alloc_top as *mut u8,
                Layout::from_size_align_unchecked(
                    self.alloc_end - self.alloc_top, 4)
            )
        }
    }

    unsafe fn __alloc(&self, size: usize) -> *mut u8 {
        if !self.initialized {
            panic!("Heap used before initialize allocator");
        }

        let mut smallest: Option<&mut Area> = None;
        for area in self.free.iter_mut() {
            if area.size < size {
                continue;
            }

            smallest = match smallest {
                None => Some(area),
                Some(s) => {
                    if area.size < s.size {
                        Some(area)
                    } else {
                        Some(s)
                    }
                }
            };
        }

        match smallest {
            None => ptr::null_mut(),
            Some(area) => {
                let ret = area.addr as *mut u8;
                area.size -= size;
                area.addr += size;
                ret
            }
        }
    }

    unsafe fn __dealloc(&self, ptr: *mut u8, size: usize) {
        if !self.initialized {
            panic!("Heap used before initialize allocator");
        }

        let mut new_area = match self.unused.take_mut() {
            None => panic!("Free list exhausted"),
            Some(area) => area,
        };

        new_area.addr = ptr as usize;
        new_area.size = size;

        let mut left: Option<&mut Area> = None;
        let mut right: Option<&mut Area> = None;

        for area in self.free.iter_mut() {
            if let Some((addr, size)) = new_area.is_overwrapping(area) {
                panic!(
                    "Double free detected: req={:08x}+{:x}, double free={:08x}+{:x}",
                    ptr as usize, size,
                    addr, size
                )
            }
            if area.right() == new_area.addr {
                left = Some(area);
            } else if area.addr == new_area.right() {
                right = Some(area);
            }
        }

        if left.is_some() && right.is_some() {
            let left = left.unwrap();
            let right = right.unwrap();
            left.size += size + right.size;
            self.unused.append(new_area);
            self.unused.append(right);
        } else if left.is_some() {
            let left = left.unwrap();
            left.size += size;
            self.unused.append(new_area);
        } else if right.is_some() {
            let right = right.unwrap();
            right.addr -= size;
            right.size += size;
            self.unused.append(new_area);
        } else {
            self.free.append(new_area);
        }
    }
}

unsafe impl Sync for LinkedListAllocator {}

unsafe impl GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.__alloc(align_up(layout.align(), layout.size()))
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.__dealloc(ptr, align_up(layout.align(), layout.size()))
    }
}
