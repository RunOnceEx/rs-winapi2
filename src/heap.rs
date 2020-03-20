//! All heap related Windows types for the current process.

/// Uses the `kernel32` heap manipulation functions.
#[cfg(not(any(winapi = "native", winapi = "syscall")))]
pub type Heap = SystemHeapKernel32;

/// Uses the `ntdll` heap manipulation functions.
#[cfg(winapi = "native")]
pub type Heap = SystemHeapNtDll;

// TODO: Add Rust heap implementation for `#[cfg(winapi = "syscall")]` and remove `cfg` attribute in `lib.rs`.

/// Allocator which uses the native process heap, stored in the process environment block.
pub struct SystemHeapKernel32 {
    /// If set to `true`, serialized access will not be used.
    pub no_serialize: bool,
    /// If set to `true`, memory is cleared on de-allocation.
    pub clear: bool
}

unsafe impl core::alloc::GlobalAlloc for SystemHeapKernel32 {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        crate::dll::kernel32::GetProcessHeap().map(|heap| crate::dll::kernel32::HeapAlloc(
            heap,
            SystemHeapFlags::new().set(SystemHeapFlag::NoSerializeAccess, self.no_serialize),
            layout.size()
        )).unwrap_or(0 as *mut _)
    }

    unsafe fn dealloc(&self, memory: *mut u8, layout: core::alloc::Layout) {
        if self.clear {
            core::ptr::write_bytes(memory, 0, layout.size());
        }

        if crate::dll::kernel32::GetProcessHeap().and_then(|heap| crate::dll::kernel32::HeapFree(
            heap,
            SystemHeapFlags::new().set(SystemHeapFlag::NoSerializeAccess, self.no_serialize),
            memory
        ).into().then_some(())).is_none() {
            alloc::alloc::handle_alloc_error(layout);
        }
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        crate::dll::kernel32::GetProcessHeap().map(|heap| crate::dll::kernel32::HeapAlloc(
            heap,
            SystemHeapFlags::new()
                .set(SystemHeapFlag::ZeroMemory, true)
                .set(SystemHeapFlag::NoSerializeAccess, self.no_serialize),
            layout.size()
        )).unwrap_or(0 as *mut _)
    }
}

/// Allocator which uses the native process heap, stored in the process environment block.
pub struct SystemHeapNtDll {
    /// If set to `true`, serialized access will not be used.
    pub no_serialize: bool,
    /// If set to `true`, memory is cleared on de-allocation.
    pub clear: bool
}

unsafe impl core::alloc::GlobalAlloc for SystemHeapNtDll {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut heap = core::mem::MaybeUninit::uninit();
        if crate::dll::ntdll::RtlGetProcessHeaps(1, heap.as_mut_ptr()) < 1 {
            return 0 as *mut _;
        }

        crate::dll::ntdll::RtlAllocateHeap(
            heap.assume_init(),
            SystemHeapFlags::new().set(SystemHeapFlag::NoSerializeAccess, self.no_serialize),
            layout.size()
        )
    }

    unsafe fn dealloc(&self, memory: *mut u8, layout: core::alloc::Layout) {
        if self.clear {
            core::ptr::write_bytes(memory, 0, layout.size());
        }

        let mut heap = core::mem::MaybeUninit::uninit();

        if crate::dll::ntdll::RtlGetProcessHeaps(1, heap.as_mut_ptr()) < 1 ||
           crate::dll::ntdll::RtlFreeHeap(
            heap.assume_init(),
            SystemHeapFlags::new().set(SystemHeapFlag::NoSerializeAccess, self.no_serialize),
            memory
        ).into().then_some(()).is_none() {
            alloc::alloc::handle_alloc_error(layout);
        }
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut heap = core::mem::MaybeUninit::uninit();
        if crate::dll::ntdll::RtlGetProcessHeaps(1, heap.as_mut_ptr()) < 1 {
            return 0 as *mut _;
        }

        crate::dll::ntdll::RtlAllocateHeap(
            heap.assume_init(),
            SystemHeapFlags::new()
                .set(SystemHeapFlag::ZeroMemory, true)
                .set(SystemHeapFlag::NoSerializeAccess, self.no_serialize),
            layout.size()
        )
    }
}

/// Official documentation: [Heap API](https://docs.microsoft.com/en-us/windows/win32/api/heapapi/).
#[repr(transparent)]
pub(crate) struct SystemHeapHandle(core::num::NonZeroUsize);

/// Official documentation: [kernel32.HeapAlloc flags](https://docs.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapalloc).
/// Official documentation: [kernel32.HeapCreate flags](https://docs.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapcreate).
#[repr(C)]
pub(crate) struct SystemHeapFlags(bitfield::BitField32);

/// Official documentation: [kernel32.HeapAlloc flags](https://docs.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapalloc).
/// Official documentation: [kernel32.HeapCreate flags](https://docs.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapcreate).
#[allow(missing_docs)]
#[repr(u8)]
pub(crate) enum SystemHeapFlag {
    NoSerializeAccess,
    #[allow(unused)]
    GenerateExceptions,
    ZeroMemory,
    #[allow(unused)]
    CreateEnableExecute = 18
}

impl SystemHeapFlags {
    /// Creates a new instance.
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self(bitfield::BitField32::new())
    }

    /// Returns a modified variant with the flag set to the specified value.
    #[inline(always)]
    pub(crate) const fn set(&self, flag: SystemHeapFlag, value: bool) -> Self {
        Self(self.0.set_bit(flag as u8, value))
    }
}