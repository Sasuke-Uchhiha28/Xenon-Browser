//! C ABI surface of xenon-core (Architecture.md 6.3).
//!
//! This is the M1.3 stub: it pins the ABI version constants that the
//! Blink host will link against. The hot-path functions
//! (`xenon_adblock_check`, `xenon_filter_check_nav`) arrive with the
//! filter engine in M3.2/M4.1 and will be the only `unsafe` boundary —
//! each with a documented safety argument. The functions below are safe
//! and pure; no pointers cross the boundary yet.

/// ABI revision of this library. The host checks it at load time and
/// refuses to run against a mismatched core.
#[no_mangle]
pub extern "C" fn xenon_core_abi_version() -> u32 {
    1
}

/// Protocol version of the JSON-RPC surface (see `xenon_core::rpc`).
#[no_mangle]
pub extern "C" fn xenon_core_protocol_version() -> u32 {
    xenon_core::rpc::PROTOCOL_VERSION as u32
}

#[cfg(test)]
mod tests {
    #[test]
    fn versions_are_stable_within_milestone_1() {
        assert_eq!(super::xenon_core_abi_version(), 1);
        assert_eq!(super::xenon_core_protocol_version(), 1);
    }
}
