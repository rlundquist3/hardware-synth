use emballoc::Allocator;

/**
 *  Wrapper around emballoc's Allocator that disables interrupts during allocation,
 * preventing deadlocks if a low-priority task is allocating while a higher-priority
 * one attempts to interrupt.
*/
struct IrqSafeAllocator<const N: usize>(Allocator<N>);

unsafe impl<const N: usize> core::alloc::GlobalAlloc for IrqSafeAllocator<N> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        cortex_m::interrupt::free(|_| unsafe { self.0.alloc(layout) })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        cortex_m::interrupt::free(|_| unsafe { self.0.dealloc(ptr, layout) })
    }
}

#[global_allocator]
static ALLOCATOR: IrqSafeAllocator<32_768> = IrqSafeAllocator(Allocator::new());
