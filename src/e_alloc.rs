use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

pub fn init_heap(size: usize) {
    let start_addr = cortex_m_rt::heap_start() as usize;
    let end_addr = start_addr.checked_add(size).expect("size too large");
    assert!(
        end_addr < 0x70_00_00_00 + 0x02_00_00_00,
        "end address out of OCTOSPI2"
    );
    unsafe { HEAP.init(start_addr, size) }
}
