//! PCR selection used as the sealing policy on every TPM backend.

/// PCR7 - Secure Boot state. Used directly by the Windows TBS backend
pub const PCR_INDEX_BOOT_STATE: u8 = 7;

#[cfg(target_os = "linux")]
use tss_esapi::interface_types::algorithm::HashingAlgorithm;
#[cfg(target_os = "linux")]
use tss_esapi::structures::{PcrSelectionList, PcrSelectionListBuilder, PcrSlot};

/// tss_esapi equivalent of [`PCR_INDEX_BOOT_STATE`], used by the Linux backend
#[cfg(target_os = "linux")]
pub const PCR_SLOTS_FOR_BOOT: [PcrSlot; 1] = [PcrSlot::Slot7];

#[cfg(target_os = "linux")]
pub fn create_pcr_selection_for_boot_state() -> tss_esapi::Result<PcrSelectionList> {
    PcrSelectionListBuilder::new()
        .with_selection(HashingAlgorithm::Sha256, &PCR_SLOTS_FOR_BOOT)
        .build()
}
