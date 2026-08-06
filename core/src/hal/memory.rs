// Mmap or HugePages abstraction
// For the Titanium Core, we pre-allocate memory to avoid heap allocations in the hot path.

pub fn allocate_huge_buffer(size_bytes: usize) -> Vec<u8> {
    // Ideally this would use libc::mmap with MAP_HUGETLB for 2MB pages.
    // To ensure cross-platform safety and avoid OS-level page faults during runtime,
    // we allocate a standard Vec and force the OS to page it in by writing to it.
    let mut buffer = vec![0; size_bytes];
    
    // Touch every page to force physical memory allocation (prevent lazy allocation)
    let page_size = 4096;
    let mut i = 0;
    while i < size_bytes {
        buffer[i] = 1;
        i += page_size;
    }
    
    // Zero it out
    buffer.fill(0);
    
    println!("Allocated {} bytes of pre-faulted contiguous memory.", size_bytes);
    buffer
}
