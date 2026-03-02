//! Tests for IMAP uid-set formatting (format_uid_set) and flag-string helpers.

use postail_project_lib::imap::flags::format_uid_set;

// ── format_uid_set ────────────────────────────────────────────────────────────

#[test]
fn empty_slice_returns_empty_string() {
    assert_eq!(format_uid_set(&[]), "");
}

#[test]
fn single_uid() {
    assert_eq!(format_uid_set(&[42]), "42");
}

#[test]
fn two_consecutive_uids() {
    assert_eq!(format_uid_set(&[1, 2]), "1:2");
}

#[test]
fn three_consecutive_uids() {
    assert_eq!(format_uid_set(&[5, 6, 7]), "5:7");
}

#[test]
fn non_consecutive_uids() {
    assert_eq!(format_uid_set(&[1, 3, 5]), "1,3,5");
}

#[test]
fn mixed_ranges_and_singles() {
    assert_eq!(format_uid_set(&[1, 2, 5, 6, 7, 10]), "1:2,5:7,10");
}

#[test]
fn unsorted_input_is_sorted() {
    assert_eq!(format_uid_set(&[7, 1, 5, 2, 6]), "1:2,5:7");
}

#[test]
fn duplicate_uids_are_deduped() {
    assert_eq!(format_uid_set(&[1, 1, 2, 2, 3]), "1:3");
}

#[test]
fn single_element_range_stays_as_single() {
    // Range of one is just the number
    assert_eq!(format_uid_set(&[100]), "100");
}

#[test]
fn large_contiguous_range() {
    let uids: Vec<u32> = (1000..=2000).collect();
    assert_eq!(format_uid_set(&uids), "1000:2000");
}

#[test]
fn three_separate_ranges() {
    // 1:3, 10:12, 20:22
    let uids = vec![1, 2, 3, 10, 11, 12, 20, 21, 22];
    assert_eq!(format_uid_set(&uids), "1:3,10:12,20:22");
}

#[test]
fn uid_1_is_first_start() {
    assert_eq!(format_uid_set(&[1, 2, 3, 100]), "1:3,100");
}

#[test]
fn alternating_pairs() {
    assert_eq!(format_uid_set(&[1, 2, 4, 5, 7, 8]), "1:2,4:5,7:8");
}

#[test]
fn same_uid_many_times() {
    assert_eq!(format_uid_set(&[99, 99, 99]), "99");
}

#[test]
fn maximum_u32_value() {
    let max = u32::MAX;
    assert_eq!(format_uid_set(&[max]), max.to_string());
}

#[test]
fn two_uids_separated_by_gap_of_two() {
    // 5 and 7 → NOT consecutive, must be "5,7"
    assert_eq!(format_uid_set(&[5, 7]), "5,7");
}

#[test]
fn produces_valid_imap_uid_set_format() {
    let result = format_uid_set(&[1, 2, 3, 10, 11, 20]);
    // Verify there are no spaces (IMAP UID sets are space-free)
    assert!(!result.contains(' '));
    // Verify only digits, colons, commas
    assert!(result
        .chars()
        .all(|c| c.is_ascii_digit() || c == ':' || c == ','));
}
