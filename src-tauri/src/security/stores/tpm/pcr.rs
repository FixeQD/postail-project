use tss_esapi::interface_types::algorithm::HashingAlgorithm;
use tss_esapi::structures::{PcrSelectionListBuilder, PcrSlot};

pub const PCR_SLOTS_FOR_BOOT: [PcrSlot; 8] = [
    PcrSlot::Slot0,
    PcrSlot::Slot1,
    PcrSlot::Slot2,
    PcrSlot::Slot3,
    PcrSlot::Slot4,
    PcrSlot::Slot5,
    PcrSlot::Slot6,
    PcrSlot::Slot7,
];

pub fn create_pcr_selection_for_boot_state(
) -> tss_esapi::Result<tss_esapi::structures::PcrSelectionList> {
    PcrSelectionListBuilder::new()
        .with_selection(HashingAlgorithm::Sha256, &PCR_SLOTS_FOR_BOOT)
        .build()
}
