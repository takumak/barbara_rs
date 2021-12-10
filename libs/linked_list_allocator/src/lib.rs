#![cfg_attr(not(test), no_std)]

use core::{
    alloc::GlobalAlloc,
    cmp::max,
    mem::align_of,
    ptr,
};
extern crate alloc;
use alloc::alloc::Layout;

const fn align_up(alignment: usize, value: usize) -> usize {
    if value % alignment != 0 {
        value + alignment - (value % alignment)
    } else {
        value
    }
}

const fn align_down(alignment: usize, value: usize) -> usize {
    if value % alignment != 0 {
        value - (value % alignment)
    } else {
        value
    }
}

struct Area {
    size: usize,
    next: *mut Area,
}

impl Area {
    const fn size_align() -> usize {
        let header_size = align_of::<Self>() as usize;
        let p2 = 1 << (usize::BITS - header_size.leading_zeros() - 1);
        if p2 < header_size {
            p2 << 1
        } else {
            p2
        }
    }

    const fn min_size() -> usize {
        Self::size_align() << 1
    }

    fn bottom(&self) -> usize {
        self.body_addr() + self.size
    }

    fn set_bottom(&mut self, bottom: usize) {
        assert!(bottom > self.body_addr());
        self.size = bottom - self.body_addr();
    }

    fn addr(&self) -> usize {
        self as *const _ as usize
    }

    fn body_addr(&self) -> usize {
        self.addr() + align_of::<Self>()
    }

    fn next(&self) -> Option<&'static mut Area> {
        if self.next.is_null() {
            None
        } else {
            Some(unsafe { &mut *self.next })
        }
    }

    fn next_addr(&self) -> usize {
        self.next as usize
    }

    fn try_union_next(&mut self) {
        if self.bottom() == self.next_addr() {
            let next = self.next().unwrap();
            self.set_bottom(next.bottom());
            self.next = next.next;
        }
    }

    fn alloc(&mut self, size: usize) -> *mut u8 {
        assert!(size <= self.size);
        assert!(size >= Self::min_size());
        assert!(size % Self::size_align() == 0);
        self.size -= size;
        self.bottom() as *mut u8
    }

    fn dealloc(&mut self, ptr: *mut u8, size: usize) {
        assert!(size >= Self::min_size());
        assert!(size % Self::size_align() == 0);

        let addr = ptr as usize;
        assert!(addr >= self.bottom());
        if !self.next.is_null() {
            assert!(addr + size <= self.next_addr());
        }

        if addr == self.bottom() {
            self.size += size;
            self.try_union_next();
        } else {
            let next_next_ptr = self.next;
            let next = unsafe { &mut *(addr as *mut Area) };
            next.set_bottom(addr + size);
            next.next = next_next_ptr;
            next.try_union_next();
            self.next = next;
        }
    }
}

struct AreaIterator {
    curr: *mut Area,
}

impl AreaIterator {
    fn new(head: &'static mut Area) -> Self {
        Self { curr: head }
    }
}

impl Iterator for AreaIterator {
    type Item = &'static mut Area;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let area = unsafe { &mut *self.curr };
            self.curr = area.next;
            Some(area)
        }
    }
}

struct AreaList {
    head: *mut Area,
}

impl AreaList {
    const fn new() -> AreaList {
        AreaList { head: ptr::null_mut() }
    }

    fn set_head(&mut self, addr: usize, size: usize) {
        self.head = addr as *mut Area;
        let head = unsafe { &mut *self.head };
        head.size = size - align_of::<Self>();
        head.next = ptr::null_mut();
    }

    fn iter_mut(&self) -> AreaIterator {
        AreaIterator::new(unsafe { &mut *self.head })
    }
}

pub struct LinkedListAllocator {
    initialized: bool,
    free_areas: AreaList,
    bottom: usize,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        LinkedListAllocator {
            initialized: false,
            free_areas: AreaList::new(),
            bottom: 0,
        }
    }

    pub fn init(&mut self, mem_top: usize, mem_bottom: usize) {
        let mem_top = align_up(Area::size_align(), mem_top);
        let mem_bottom = align_down(Area::size_align(), mem_bottom);

        if mem_bottom <= mem_top {
            panic!("Invalid heap area: top={:p} >= bottom={:p}",
                   mem_top as *const u8, mem_bottom as *const u8);
        }

        let mem_size = mem_bottom - mem_top;

        let min_size = Area::size_align() * 100;
        if mem_size < min_size {
            panic!("Heap area too small: given={:#08x}, required={:#08x}",
                   mem_size, min_size);
        }

        self.initialized = true;
        self.free_areas.set_head(mem_top, mem_size);
        self.bottom = mem_bottom;
    }

    #[cfg(test)]
    fn total_free(&self) -> usize {
        let mut size: usize = 0;
        for area in self.free_areas.iter_mut() {
            size += area.size;
        }
        size
    }

    unsafe fn __alloc(&self, size: usize) -> *mut u8 {
        if !self.initialized {
            panic!("Heap used before initialize allocator");
        }

        let size = max(
            Area::min_size(),
            align_up(Area::size_align(), size));

        let mut smallest: Option<&mut Area> = None;
        for area in self.free_areas.iter_mut() {
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
                area.alloc(size)
            }
        }
    }

    unsafe fn __dealloc(&self, ptr: *mut u8, size: usize) {
        if !self.initialized {
            panic!("Heap used before initialize allocator");
        }

        let addr = ptr as usize;
        let size = max(
            Area::min_size(),
            align_up(Area::size_align(), size));

        assert!(addr + size <= self.bottom);

        let mut area: Option<&mut Area> = None;
        for a in self.free_areas.iter_mut() {
            let a_addr = a.addr();
            if a_addr > addr {
                break;
            }
            area = Some(a);
        }

        match area {
            None => panic!("Invalid pointer"),
            Some(area) => area.dealloc(ptr, size),
        };
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

#[cfg(test)]
mod tests {
    extern crate mersenne_twister;
    extern crate rand;

    use crate::LinkedListAllocator;

    const TEST_HEAP_SIZE: usize = 0x10000;

    struct Heap {
        allocator: LinkedListAllocator,
        buffer: [u8; TEST_HEAP_SIZE],
    }

    impl Heap {
        fn new() -> Self {
            let mut i = Self {
                allocator: LinkedListAllocator::new(),
                buffer: [0; TEST_HEAP_SIZE],
            };
            i.allocator.init(i.buf_top(), i.buf_end());
            i
        }

        fn buf_top(&self) -> usize {
            self.buffer.as_ptr() as usize
        }

        fn buf_end(&self) -> usize {
            self.buf_top() + self.buf_size()
        }

        fn buf_size(&self) -> usize {
            self.buffer.len()
        }

        fn check_in_range(&self, addr: usize, size: usize) {
            assert!(self.buf_top() <= addr && addr + size <= self.buf_end());
        }

        #[cfg(test)]
        fn total_free(&self) -> usize {
            self.allocator.total_free()
        }

        fn alloc(&self, size: usize) -> *mut u8 {
            let ptr = unsafe { self.allocator.__alloc(size) };
            if !ptr.is_null() {
                self.check_in_range(ptr as usize, size);
            }
            ptr
        }

        fn dealloc(&self, ptr: *mut u8, size: usize) {
            unsafe { self.allocator.__dealloc(ptr, size) }
        }
    }

    #[test]
    #[should_panic]
    fn before_init() {
        let heap = LinkedListAllocator::new();
        unsafe {
            heap.__alloc(0x1000);
        }
    }

    #[test]
    fn alloc_free() {
        let heap = Heap::new();
        let ptr = heap.alloc(0x1000);
        heap.dealloc(ptr, 0x1000);
    }

    #[test]
    fn alloc_free2() {
        let heap = Heap::new();
        let total_free = heap.total_free();

        let ptr1 = heap.alloc(4);
        let ptr2 = heap.alloc(8);
        let ptr3 = heap.alloc(12);
        let ptr4 = heap.alloc(16);
        let ptr5 = heap.alloc(20);
        heap.dealloc(ptr2, 8);
        heap.dealloc(ptr4, 16);
        heap.dealloc(ptr3, 12);
        heap.dealloc(ptr1, 4);
        heap.dealloc(ptr5, 20);

        assert!(heap.total_free() == total_free);
    }

    #[test]
    fn random() {
        use mersenne_twister::MersenneTwister;
        use rand::{Rng, SeedableRng};

        let heap = Heap::new();
        let total_free = heap.total_free();

        let seed: u64 = 0xea0a_b58a_f23d_9521;
        let mut rng: MersenneTwister = SeedableRng::from_seed(seed);

        for _ in 0..100 {
            let mut size_list: Vec<usize> =
                (4..(TEST_HEAP_SIZE/2)).step_by(4).collect();
            let mut free_list: Vec<(*mut u8, usize)> = Vec::new();

            rng.shuffle(&mut size_list);
            for size in size_list {
                let ptr = heap.alloc(size);
                if ptr.is_null() {
                    break
                }
                free_list.push((ptr, size));
            }

            rng.shuffle(&mut free_list);
            for (ptr, size) in free_list {
                heap.dealloc(ptr, size);
                heap.total_free();
            }

            assert!(heap.total_free() == total_free);
        }
    }
}
