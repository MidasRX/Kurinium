use std::env;
use sysinfo::{System, SystemExt, ProcessExt, PidExt};
use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE};
use winapi::um::handleapi::CloseHandle;

// checks if any process is bypassed. needs admin rights to check other admin procs.
fn is_proc_elevated(pid: u32) -> bool {
    use winapi::um::processthreadsapi::OpenProcessToken;
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use std::mem;

    unsafe {
        let proc_handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
        if proc_handle.is_null() {
            return false; // cannot open proc, assume not bypass
        }

        let mut token_handle = mem::zeroed();
        let success = OpenProcessToken(proc_handle, TOKEN_QUERY, &mut token_handle) != 0;

        let mut is_elevated = false;
        if success {
            let mut elevation: TOKEN_ELEVATION = mem::zeroed();
            let mut size = mem::size_of::<TOKEN_ELEVATION>() as u32;
            let success = GetTokenInformation(
                token_handle,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size,
                &mut size,
            ) != 0;
            if success {
                is_elevated = elevation.TokenIsElevated != 0;
            }
        }

        if !token_handle.is_null() {
            CloseHandle(token_handle);
        }
        CloseHandle(proc_handle);
        
        is_elevated
    }
}

// find and kill other instances based on privilege
pub fn singleton_prcess(current_is_admin: bool) {
    let Ok(current_exe_path) = env::current_exe() else { return; };
    let Some(current_exe_name) = current_exe_path.file_name().and_then(|n| n.to_str()) else { return; };
    let current_pid = std::process::id();

    let s = System::new_with_specifics(sysinfo::RefreshKind::new().with_processes(sysinfo::ProcessRefreshKind::new()));

    for process in s.processes_by_name(current_exe_name) {
        let pid = process.pid().as_u32();
        if pid == current_pid {
            continue; // dont kill self
        }

        let other_is_admin = is_proc_elevated(pid);

        if current_is_admin {
            // admin kills everyone
            unsafe {
                let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
                if !handle.is_null() {
                    TerminateProcess(handle, 1);
                    CloseHandle(handle);
                }
            }
        } else {
            // user sees an admin, user must die
            if other_is_admin {
                std::process::exit(0);
            } else {
                // user sees another user, kill it
                 unsafe {
                    let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
                    if !handle.is_null() {
                        TerminateProcess(handle, 1);
                        CloseHandle(handle);
                    }
                }
            }
        }
    }
}