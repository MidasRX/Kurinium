use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use std::env;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
use winapi::um::winnt::PROCESS_TERMINATE;

use crate::config::Config;
use crate::utils::admin::is_process_elevated;

pub fn singleton_prcess(current_is_admin: bool) {
    let Ok(current_exe_path) = env::current_exe() else {
        return;
    };
    let current_pid = std::process::id();

    let mut names_to_check = std::collections::HashSet::new();
    if let Some(name) = current_exe_path.file_name().and_then(|n| n.to_str()) {
        names_to_check.insert(name.to_string());
    }
    names_to_check.insert(Config::get_exe_name().to_string());

    let s = System::new_with_specifics(
        sysinfo::RefreshKind::new().with_processes(sysinfo::ProcessRefreshKind::new()),
    );

    for name in names_to_check {
        for process in s.processes_by_name(&name) {
            let pid = process.pid().as_u32();
            if pid == current_pid {
                continue; // Don't kill self
            }

            let other_is_admin = is_process_elevated(pid);

            if current_is_admin {
                // Admin kills everyone
                terminate_process(pid);
            } else {
                // Non-admin sees an admin, non-admin must die
                if other_is_admin {
                    std::process::exit(0);
                } else {
                    // Non-admin sees another non-admin, kill it
                    terminate_process(pid);
                }
            }
        }
    }
}

// Terminate a process by PID
fn terminate_process(pid: u32) {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if !handle.is_null() {
            TerminateProcess(handle, 1);
            CloseHandle(handle);
        }
    }
}
