use kernel_api::{OsError, OsResult};
use shim::{io, ioerr};
use crate::console::kprintln;
use crate::memory::{PagePermissions, VirtualAddr};
use crate::param::USER_IMG_BASE;
use crate::SCHEDULER;
use crate::traps::syndrome::{AbortData, FaultStatusCode};
use crate::traps::TrapFrame;

pub(crate) fn handle_memory_abort(trap_frame: &mut TrapFrame, abort_data: AbortData) -> OsResult<()> {
    let faulting_address = VirtualAddr::from(unsafe { aarch64::FAR_EL1.get() }).page_aligned();
    let AbortData { write, fault_status_code } = &abort_data;

    info!("trap at {:x}", trap_frame.elr);

    if !write {
        return Err(OsError::Unknown);
    }

    match fault_status_code {
        FaultStatusCode::PermissionFault3 => {
            SCHEDULER.on_process(trap_frame, |process| {
                process.vmap.remove_cow(faulting_address, process.context.tpidr)
            })?
        }
        _ => return Err(OsError::Unknown),
    }
}