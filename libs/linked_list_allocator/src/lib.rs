#![cfg_attr(not(test), no_std)]
#![feature(no_coverage)]

use core::{alloc::GlobalAlloc, cell::UnsafeCell, mem::size_of, ptr};
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
        let header_size = size_of::<Self>();
        let p2 = 1 << (usize::BITS - header_size.leading_zeros() - 1);
        align_up(p2, header_size)
    }

    fn addr(&self) -> usize {
        self as *const _ as usize
    }

    fn bottom(&self) -> usize {
        self.addr() + self.size
    }

    fn set_bottom(&mut self, bottom: usize) {
        assert!(bottom > self.addr());
        self.size = bottom - self.addr();
    }
}

struct AreaIterator {
    curr: *mut Area,
    prev: *mut Area,
}

impl AreaIterator {
    fn new(head: *mut Area) -> Self {
        Self {
            curr: head,
            prev: ptr::null_mut(),
        }
    }
}

impl Iterator for AreaIterator {
    type Item = (&'static mut Area, Option<&'static mut Area>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let curr = unsafe { &mut *self.curr };
            let prev = if self.prev.is_null() {
                None
            } else {
                Some(unsafe { &mut *self.prev })
            };
            self.curr = curr.next;
            self.prev = curr;
            Some((curr, prev))
        }
    }
}

struct AreaList {
    head: *mut Area,
}

impl AreaList {
    const fn new() -> AreaList {
        AreaList {
            head: ptr::null_mut(),
        }
    }

    fn init(&mut self, addr: usize, size: usize) {
        self.head = addr as *mut Area;
        let head = unsafe { &mut *self.head };
        head.size = size;
        head.next = ptr::null_mut();
    }

    fn iter_with_prev(&self) -> AreaIterator {
        AreaIterator::new(self.head)
    }
}

pub struct LinkedListAllocator {
    initialized: bool,
    free_areas: UnsafeCell<AreaList>,
    mem_top: usize,
    mem_end: usize,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        LinkedListAllocator {
            initialized: false,
            free_areas: UnsafeCell::new(AreaList::new()),
            mem_top: 0,
            mem_end: 0,
        }
    }

    pub fn init(&mut self, mem_top: usize, mem_end: usize) {
        let mem_top = align_up(Area::size_align(), mem_top);
        let mem_end = align_down(Area::size_align(), mem_end);

        if mem_end <= mem_top {
            panic!(
                "Invalid heap area: top={:p} >= bottom={:p}",
                mem_top as *const u8, mem_end as *const u8
            );
        }

        let mem_size = mem_end - mem_top;

        let min_size = Area::size_align() * 100;
        if mem_size < min_size {
            panic!(
                "Heap area too small: given={:#08x}, required={:#08x}",
                mem_size, min_size
            );
        }

        self.initialized = true;
        self.free_areas.get_mut().init(mem_top, mem_size);
        self.mem_top = mem_top;
        self.mem_end = mem_end;
    }

    unsafe fn __alloc(&self, size: usize) -> *mut u8 {
        if !self.initialized {
            panic!("Heap used before initialize allocator");
        }

        let size = align_up(Area::size_align(), size);

        let list = &mut *self.free_areas.get();

        let mut target: Option<&'static mut Area> = None;
        let mut target_size: usize = 0;
        let mut target_prev: Option<&'static mut Area> = None;
        for (area, prev) in list.iter_with_prev() {
            let area_size = area.size;
            if area_size < size {
                continue;
            }

            if target.is_none() || area_size < target_size {
                target_size = area_size;
                target = Some(area);
                target_prev = prev;
            }
        }

        if target.is_none() {
            return ptr::null_mut();
        }

        let target = target.unwrap();
        let result = target.addr() as *mut u8;
        let prev_next: *mut Area = if size == target_size {
            target.next
        } else {
            let next = &mut *((target.addr() + size) as *mut Area);
            next.set_bottom(target.bottom());
            next.next = target.next;
            next
        };

        if target_prev.is_none() {
            list.head = prev_next;
        } else {
            target_prev.unwrap().next = prev_next;
        }

        result
    }

    unsafe fn __dealloc(&self, ptr: *mut u8, size: usize) {
        if !self.initialized {
            panic!("Heap used before initialize allocator");
        }

        let addr = ptr as usize;
        let size = align_up(Area::size_align(), size);

        assert!(addr >= self.mem_top);
        assert!(addr + size <= self.mem_end);

        let mut area = &mut *(addr as *mut Area);
        area.set_bottom(addr + size);
        area.next = ptr::null_mut();

        let list = &mut *self.free_areas.get();
        if list.head.is_null() {
            list.head = area;
            return;
        }

        let mut target: Option<&'static mut Area> = None;
        for (area, _) in list.iter_with_prev() {
            if area.addr() > addr {
                break;
            }
            target = Some(area);
        }

        if target.is_some() {
            let target = target.unwrap();
            if target.bottom() == area.addr() {
                target.set_bottom(area.bottom());
                area = target;
            } else {
                area.next = target.next;
                target.next = area;
            }
        } else {
            area.next = list.head;
            list.head = area;
        }

        if !area.next.is_null() {
            let next = &mut *area.next;
            if area.bottom() == next.addr() {
                area.set_bottom(next.bottom());
                area.next = next.next;
            }
        }
    }
}

unsafe impl Sync for LinkedListAllocator {}

unsafe impl GlobalAlloc for LinkedListAllocator {
    #[no_coverage]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.__alloc(align_up(layout.align(), layout.size()))
    }

    #[no_coverage]
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

        fn total_free(&mut self) -> usize {
            let mut size: usize = 0;
            let list = self.allocator.free_areas.get_mut();
            for (area, _) in list.iter_with_prev() {
                size += area.size;
            }
            size
        }

        fn is_in_mem_range(&self, addr: usize) -> bool {
            self.allocator.mem_top <= addr && addr < self.allocator.mem_end
        }

        #[no_coverage]
        fn check_free_areas(&mut self, print: bool) -> bool {
            if print {
                println!("----");
            }

            let mut ok = true;
            let list = self.allocator.free_areas.get_mut();

            let mut area = list.head;
            while !area.is_null() {
                if !self.is_in_mem_range(area as usize) {
                    if print {
                        println!("  {:p}!out_of_range!", area);
                    }
                    ok = false;
                    break;
                }

                let size = unsafe { (*area).size };
                let next = unsafe { (*area).next };

                if print {
                    print!(
                        "  top+{:04x}(size={:x}",
                        area as usize - self.allocator.mem_top,
                        size
                    );
                }

                if area as usize + size > self.allocator.mem_end {
                    if print {
                        print!("!out_of_range!");
                    }
                    ok = false;
                }

                if !next.is_null() && area as usize + size > next as usize {
                    if print {
                        print!("!overlap_next!");
                    }
                    ok = false;
                }

                if print {
                    print!(") -> ");
                }

                if next.is_null() {
                    if print {
                        println!("null");
                    }
                } else if self.is_in_mem_range(next as usize) {
                    if print {
                        println!("top+{:04x}", next as usize - self.allocator.mem_top);
                    }
                } else {
                    if print {
                        println!("{:p}!out_of_range!", next);
                    }
                    ok = false;
                }

                if !ok {
                    break;
                }

                area = next;
            }

            if print {
                println!("----");
            }

            ok
        }

        #[allow(dead_code)]
        #[no_coverage]
        fn print_free_areas(&mut self) {
            self.check_free_areas(true);
        }

        #[no_coverage]
        fn check_integrity(&mut self) {
            if !self.check_free_areas(false) {
                self.check_free_areas(true);
                panic!("[Bug] Broken linked list");
            }
        }

        fn check_in_range(&self, addr: usize, size: usize) {
            assert!(self.buf_top() <= addr && addr + size <= self.buf_end());
        }

        fn alloc(&mut self, size: usize) -> *mut u8 {
            let ptr = unsafe { self.allocator.__alloc(size) };
            if !ptr.is_null() {
                self.check_in_range(ptr as usize, size);
            }
            self.check_integrity();
            ptr
        }

        fn dealloc(&mut self, ptr: *mut u8, size: usize) {
            unsafe { self.allocator.__dealloc(ptr, size) }
            self.check_integrity();
        }
    }

    #[test]
    #[should_panic]
    fn before_init_1() {
        let heap = LinkedListAllocator::new();
        unsafe {
            heap.__alloc(0x1000);
        }
    }

    #[test]
    #[should_panic]
    fn before_init_2() {
        let heap = LinkedListAllocator::new();
        let buf: [u8; 0x10] = [0; 0x10];
        unsafe {
            heap.__dealloc(buf.as_ptr() as usize as *mut u8, 0x10);
        }
    }

    #[test]
    fn alloc_free() {
        let mut heap = Heap::new();
        let ptr = heap.alloc(0x1000);
        heap.dealloc(ptr, 0x1000);
    }

    #[test]
    fn alloc_free2() {
        let mut heap = Heap::new();
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

        let mut heap = Heap::new();
        let total_free = heap.total_free();

        let seed: u64 = 0xea0a_b58a_f23d_9521;
        let mut rng: MersenneTwister = SeedableRng::from_seed(seed);
        let mut free_list: Vec<(*mut u8, usize)> = Vec::new();

        for _ in 0..100 {
            loop {
                let size = rng.gen_range(1, TEST_HEAP_SIZE / 128) * 4;
                let ptr = heap.alloc(size);
                if ptr.is_null() {
                    break;
                }
                free_list.push((ptr, size));
            }

            rng.shuffle(&mut free_list);
            for _ in 0..rng.gen_range(0, free_list.len()) {
                let (ptr, size) = free_list.pop().unwrap();
                heap.dealloc(ptr, size);
            }
        }

        let size = crate::Area::size_align();
        for _ in 0..(heap.total_free() / size) {
            let ptr = heap.alloc(size);
            assert!(!ptr.is_null());
            free_list.push((ptr, size));
        }

        rng.shuffle(&mut free_list);
        for (ptr, size) in free_list {
            heap.dealloc(ptr, size);
        }

        assert!(heap.total_free() == total_free);
    }

    #[test]
    #[should_panic]
    fn invalid_address() {
        let mut allocator = LinkedListAllocator::new();
        let buf: [u8; 0x1000] = [0; 0x1000];
        allocator.init(buf.as_ptr() as usize, buf.as_ptr() as usize);
    }

    #[test]
    #[should_panic]
    fn region_size_1() {
        let mut allocator = LinkedListAllocator::new();
        let buf: [u8; 0x1000] = [0; 0x1000];
        let addr: usize = buf.as_ptr() as usize;
        allocator.init(addr, addr + (core::mem::size_of::<usize>() * 2 * 99));
    }

    #[test]
    fn region_size_2() {
        let mut allocator = LinkedListAllocator::new();
        let buf: [u8; 0x1000] = [0; 0x1000];
        let addr: usize = buf.as_ptr() as usize;
        allocator.init(addr, addr + (core::mem::size_of::<usize>() * 2 * 100));
    }
}
