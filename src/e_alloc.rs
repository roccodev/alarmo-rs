use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

pub fn init_heap(size: usize) {
    let end_addr = 0x71_ff_ff_ff_usize;
    let start_addr = end_addr.checked_sub(size).expect("size too large") + 1;
    assert!(start_addr >= 0x70_00_00_00, "start address out of OCTOSPI2");
    unsafe { HEAP.init(start_addr, size) }
}

#[no_mangle]
pub fn __aeabi_unwind_cpp_pr0() {
    // TODO: what is this? It won't link if not defined
    loop {}
}
