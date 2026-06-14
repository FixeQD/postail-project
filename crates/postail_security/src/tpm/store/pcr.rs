use tss_esapi::interface_types::algorithm::HashingAlgorithm;
use tss_esapi::structures::{PcrSelectionListBuilder, PcrSlot};

pub const PCR_SLOTS_FOR_BOOT: [PcrSlot; 1] = [
    PcrSlot::Slot7,
];

pub fn create_pcr_selection_for_boot_state(
) -> tss_esapi::Result<tss_esapi::structures::PcrSelectionList> {
    PcrSelectionListBuilder::new()
        .with_selection(HashingAlgorithm::Sha256, &PCR_SLOTS_FOR_BOOT)
        .build()
}
