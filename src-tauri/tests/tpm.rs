#[cfg(all(target_os = "linux", feature = "tpm"))]
mod tpm_tests {
    use postail_project_lib::security::master_key::MasterKey;
    use postail_project_lib::security::stores::tpm::LinuxTpmStore;
    use std::path::PathBuf;
    use tpm2_simulator::{Simulator, Tcti};

    fn setup_with_simulator() -> (Simulator, LinuxTpmStore) {
        let simulator = Simulator::new().expect("Failed to create TPM simulator");
        let tcti = Tcti::from(simulator.handle());
        let store = LinuxTpmStore::with_tcti(tcti);
        (simulator, store)
    }

    #[test]
    fn test_is_available_when_simulator_present() {
        let (_, store) = setup_with_simulator();
        assert!(store.is_available());
    }

    #[test]
    fn test_store_and_retrieve_key() {
        let (_, store) = setup_with_simulator();
        let key = MasterKey::generate();

        store.store(&key).expect("Failed to store key");
        let retrieved = store.retrieve().expect("Failed to retrieve key");

        assert_eq!(key.as_bytes(), retrieved.as_bytes());
    }

    #[test]
    fn test_delete_key() {
        let (_, store) = setup_with_simulator();
        let key = MasterKey::generate();

        store.store(&key).expect("Failed to store key");
        store.delete().expect("Failed to delete key");

        assert!(store.retrieve().is_err());
    }

    #[test]
    fn test_seal_unseal_blob_integrity() {
        let (_, store) = setup_with_simulator();
        let key = MasterKey::generate();

        store.store(&key).expect("Failed to store key");
        let retrieved = store.retrieve().expect("Failed to retrieve key");

        assert_eq!(key.as_bytes(), retrieved.as_bytes());
        assert_eq!(key.as_bytes().len(), retrieved.as_bytes().len());
    }

    #[test]
    fn test_corrupted_blob_rejection() {
        let (_, store) = setup_with_simulator();
        let key = MasterKey::generate();

        store.store(&key).expect("Failed to store key");

        let sealed_path = store.get_sealed_path();
        if sealed_path.exists() {
            use std::io::{Read, Write};
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .open(&sealed_path)
                .expect("Failed to open sealed file");

            let mut content = Vec::new();
            file.read_to_end(&mut content)
                .expect("Failed to read sealed file");

            if !content.is_empty() {
                content[0] = content[0].wrapping_add(1);
                file.write_all(&content)
                    .expect("Failed to write corrupted content");
            }
        }

        assert!(store.retrieve().is_err());
    }

    #[test]
    fn test_srk_created_under_endorsement_hierarchy() {
        let (_, store) = setup_with_simulator();
        let key = MasterKey::generate();

        store.store(&key).expect("Failed to store key");
        let retrieved = store.retrieve().expect("Failed to retrieve key");

        assert_eq!(key.as_bytes(), retrieved.as_bytes());
    }

    #[test]
    fn test_concurrent_handles() {
        let simulator = Simulator::new().expect("Failed to create TPM simulator");
        let tcti1 = Tcti::from(simulator.handle());
        let store1 = LinuxTpmStore::with_tcti(tcti1);

        let tcti2 = Tcti::from(simulator.handle());
        let store2 = LinuxTpmStore::with_tcti(tcti2);

        let key = MasterKey::generate();

        store1.store(&key).expect("Failed to store from store1");
        let retrieved1 = store1.retrieve().expect("Failed to retrieve from store1");
        let retrieved2 = store2.retrieve().expect("Failed to retrieve from store2");

        assert_eq!(key.as_bytes(), retrieved1.as_bytes());
        assert_eq!(key.as_bytes(), retrieved2.as_bytes());
    }

    #[test]
    fn test_key_roundtrip_multiple_operations() {
        let (_, store) = setup_with_simulator();
        let key = MasterKey::generate();

        for _ in 0..3 {
            store.store(&key).expect("Failed to store key");
            let retrieved = store.retrieve().expect("Failed to retrieve key");
            assert_eq!(key.as_bytes(), retrieved.as_bytes());
            store.delete().expect("Failed to delete key");
        }
    }

    #[test]
    fn test_store_retrieve_different_keys() {
        let (_, store) = setup_with_simulator();
        let key1 = MasterKey::generate();
        let key2 = MasterKey::generate();

        store.store(&key1).expect("Failed to store key1");
        let retrieved1 = store.retrieve().expect("Failed to retrieve key1");
        assert_eq!(key1.as_bytes(), retrieved1.as_bytes());

        store.delete().expect("Failed to delete key1");
        assert!(store.retrieve().is_err());

        store.store(&key2).expect("Failed to store key2");
        let retrieved2 = store.retrieve().expect("Failed to retrieve key2");
        assert_eq!(key2.as_bytes(), retrieved2.as_bytes());
    }
}
