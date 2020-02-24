//! Direct implementation for the exports by several Windows dynamically loaded libraries.

/// Table that stores all system call ids which might get used by the internal functions.
#[allow(missing_docs)]
pub struct Ids {
    pub close: u16,
    pub terminate_process: u16
}

/// Global instance of system call ids that is used by all internal functions.
pub static mut IDS: Option<Ids> = None;

// TODO: Add x86 assembly shell code variant.
// TODO: Handle more than 4 parameters.
// TODO: Handle WoW64?
#[cfg(target_arch = "x86_64")]
macro_rules! syscall {
    ($id:ident) => {
        syscall!($id,)
    };
    ($id:ident, $p1:ident) => {
        syscall!($id, "{rcx}"($p1))
    };
    ($id:ident, $p1:ident, $p2:ident) => {
        syscall!($id, "{rcx}"($p1), "{rdx}"($p2))
    };
    ($id:ident, $p1:ident, $p2:ident, $p3:ident) => {
        syscall!($id, "{rcx}"($p1), "{rdx}"($p2), "{r8}"($p3))
    };
    ($id:ident, $p1:ident, $p2:ident, $p3:ident, $p4:ident) => {
        syscall!($id, "{rcx}"($p1), "{rdx}"($p2), "{r8}"($p3), "{r9}"($p4))
    };
    ($id:ident, $($input:tt)*) => {{
        let index = match IDS {
            Some(ref ids) => ids.$id,
            None => return crate::error::NtStatusValue::InvalidSystemService.into()
        } as usize;

        let result: u32;
        asm!(
            "   mov r10, rcx
                syscall
            " :
            "={eax}"(result) :
            "{eax}"(index), $($input)* :
            "r10" :
            "intel", "volatile"
        );

        *(&result as *const _ as *const crate::error::NtStatus)
    }};
}

/// Official documentation: [ntdll.NtClose](https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/nf-ntifs-ntclose).
#[allow(non_snake_case)]
#[inline(always)]
pub(crate) unsafe fn NtClose(
    object: crate::object::Handle
) -> crate::error::NtStatus {
    let object = *(&object as *const _ as *const isize);

    syscall!(close, object)
}

/// Official documentation: [ntdll.NtTerminateProcess](https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nf-ntddk-zwterminateprocess).
#[allow(non_snake_case)]
#[inline(always)]
pub(crate) unsafe fn NtTerminateProcess(
    process: crate::object::Handle,
    exit_code: u32
) -> crate::error::NtStatus {
    let process = *(&process as *const _ as *const isize);

    syscall!(terminate_process, process, exit_code)
}