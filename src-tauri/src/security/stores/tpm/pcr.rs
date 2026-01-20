use tss_esapi::interface_types::algorithm::HashingAlgorithm;
use tss_esapi::structures::{PcrSelectionListBuilder, PcrSlot};

pub const PCR_SLOTS_FOR_BOOT: [PcrSlot; 1] = [
    // PcrSlot::Slot0, //-> Core BIOS/UEFI firmware
    // PcrSlot::Slot1, //-> Platform configuration
    // PcrSlot::Slot2, //-> UEFI drivers (e.g. GPU)
    // PcrSlot::Slot3, //-> Extension config
    // PcrSlot::Slot4, //-> Bootloader code
    // PcrSlot::Slot5, //-> Partition table
    // PcrSlot::Slot6, //-> State change events
    PcrSlot::Slot7,    //-> Secure Boot state
];

pub fn create_pcr_selection_for_boot_state(
) -> tss_esapi::Result<tss_esapi::structures::PcrSelectionList> {
    PcrSelectionListBuilder::new()
        .with_selection(HashingAlgorithm::Sha256, &PCR_SLOTS_FOR_BOOT)
        .build()
}
