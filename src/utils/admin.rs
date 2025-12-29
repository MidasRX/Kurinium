use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcess, OpenProcessToken};
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{
    TokenElevation, PROCESS_QUERY_INFORMATION, TOKEN_ELEVATION, TOKEN_QUERY,
};

pub fn is_admin() -> bool {
    unsafe {
        let mut token_handle = std::mem::zeroed();
        let current_process = GetCurrentProcess();

        if OpenProcessToken(current_process, TOKEN_QUERY, &mut token_handle) != 0 {
            let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
            let mut size = std::mem::size_of_val(&elevation) as u32;

            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size,
                &mut size,
            );

            CloseHandle(token_handle);

            if result != 0 {
                return elevation.TokenIsElevated != 0;
            }
        }
        false
    }
}

pub fn is_process_elevated(pid: u32) -> bool {
    unsafe {
        let proc_handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
        if proc_handle.is_null() {
            return false; // Cannot open process, assume not elevated
        }

        let mut token_handle = std::mem::zeroed();
        let success = OpenProcessToken(proc_handle, TOKEN_QUERY, &mut token_handle) != 0;

        let mut is_elevated = false;
        if success {
            let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
            let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size,
                &mut size,
            );
            if result != 0 {
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
