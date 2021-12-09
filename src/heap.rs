extern crate linked_list_allocator;
extern crate alloc;
use alloc::alloc::Layout;

use crate::decl_c_symbol_addr;
decl_c_symbol_addr!(__heap_s, heap_s);
decl_c_symbol_addr!(__heap_e, heap_e);

use linked_list_allocator::LinkedListAllocator;
#[global_allocator]
static mut HEAP: LinkedListAllocator = LinkedListAllocator::new();

pub fn init() {
    unsafe { HEAP.init(heap_s(), heap_e()) };
}

#[alloc_error_handler]
fn alloc_error(_: Layout) -> ! {
    panic!("OOM error");
}
