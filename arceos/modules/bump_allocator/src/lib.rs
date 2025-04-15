#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    /// Start address of the memory range
    start: usize,
    /// End address of the memory range
    end: usize,
    /// Current position of the bytes area
    b_pos: usize,
    /// Current position of the pages area
    p_pos: usize,
    /// Number of bytes used
    count: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        // Initialize the allocator with the given start address and size
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = start + size;
        self.count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unimplemented!()
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        // Check if there is enough space for the requested layout
        let size = layout.size();
        if self.b_pos + size > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        // Allocate memory at the current position
        let pos = NonNull::new(self.b_pos as *mut u8).ok_or(AllocError::NoMemory)?;
        self.b_pos += size;
        self.count += 1;

        Ok(pos)
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        // Deallocate memory at the given position
        let pos = pos.as_ptr() as usize;
        if pos >= self.start && pos < self.end {
            self.count -= 1;
            if self.count == 0 {
                // Free the bytes-used area
                self.b_pos = pos;
            }
        } else {
            panic!("Invalid bytes deallocation!");
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.end - self.b_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        // Check if the requested number of pages can be allocated
        if self.p_pos - num_pages * PAGE_SIZE < self.start {
            return Err(AllocError::NoMemory);
        }

        // Align the position to the requested alignment
        let aligned_pos = (self.p_pos - num_pages * PAGE_SIZE) & !(align_pow2 - 1);

        // Update the position and return the allocated address
        self.p_pos = aligned_pos;
        Ok(aligned_pos)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        // Deallocate pages at the given position
        if pos >= self.start && pos < self.end {
            // Do nothing, as pages are not freed in this allocator
        } else {
            panic!("Invalid pages deallocation!");
        }
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.p_pos - self.start) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.end - self.p_pos) / PAGE_SIZE
    }
}
