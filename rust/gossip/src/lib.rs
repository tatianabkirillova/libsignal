use libsignal_keytrans::{
    FullTreeHead, LastTreeHead, KeyTransparency
};
use prost::Message;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub mod proto {
    tonic::include_proto!("gossip");
}

pub struct Gossiper {
    storage: GossipStorage,
    kt: Option<KeyTransparency>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GossipError {
    Invalid,
    Inconsistent
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gossip {
    pub full_tree_head : FullTreeHead,    
    pub origin_id : Vec<u8>, // who originated this gossip
    pub consistency_proof: Vec<Vec<u8>>,
}

pub struct GossipStorage {
    head: Option<FullTreeHead>,
}

static GLOBAL_GOSSIP_STORAGE: Lazy<Mutex<GossipStorage>> = Lazy::new(|| {
    Mutex::new(GossipStorage::new())
});

static GLOBAL_GOSSIPER: Lazy<Mutex<Option<Gossiper>>> = Lazy::new(|| {
    Mutex::new(None)
});

pub fn init_gossiper(kt: KeyTransparency) {
    let mut guard = GLOBAL_GOSSIPER
        .lock()
        .expect("GLOBAL_GOSSIPER lock");
    if guard.is_none() {
        let storage = GossipStorage::new();
        *guard = Some(Gossiper::new(storage, Some(kt)));
    }
}

/*
Usage:
init_gossiper(kt_config);
if let Some(g) = gossiper().as_ref() {
    let msg_with_gossip = g.append_gossip(message_bytes, origin_id);
}
*/
pub fn gossiper() -> std::sync::MutexGuard<'static, Option<Gossiper>> {
    GLOBAL_GOSSIPER
        .lock()
        .expect("GLOBAL_GOSSIPER lock")
}

impl Gossip {
    pub fn new(
        full_tree_head: FullTreeHead,
        origin_id: Vec<u8>,
        consistency_proof: Vec<Vec<u8>>
    ) -> Self {
        Self {
            full_tree_head,
            origin_id,
            consistency_proof
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, GossipError> {
        let mut th_bytes = Vec::new();
        self.full_tree_head
            .encode(&mut th_bytes)
            .map_err(|_| GossipError::Invalid)?;

        let proto = proto::Gossip {
            full_tree_head: th_bytes,
            origin_id: self.origin_id.clone(),
            consistency_proof: self.consistency_proof.clone(),
        };

        let mut out = Vec::with_capacity(proto.encoded_len());
        proto
            .encode(&mut out)
            .map_err(|_| GossipError::Invalid)?;
        Ok(out)
    }

    pub fn decode(data: &[u8]) -> Result<Self, GossipError> {
        let proto: proto::Gossip = proto::Gossip::decode(data).map_err(|_| GossipError::Invalid)?;

        let full_tree_head =
            FullTreeHead::decode(proto.full_tree_head.as_slice()).map_err(|_| GossipError::Invalid)?;

        Ok(Self {
            full_tree_head,
            origin_id: proto.origin_id,
            consistency_proof: proto.consistency_proof,
        })
    }

    pub fn verify(
        &self,
        kt: Option<&KeyTransparency>,
        last_tree_head: &LastTreeHead,
        last_distinguished: Option<&LastTreeHead>,
    ) -> Result<(), GossipError> {
        if let (Some(kt), Some(last_distinguished_)) = (kt, last_distinguished) {
            kt.verify_distinguished(
                &self.full_tree_head,
                Some(last_tree_head),
                last_distinguished_
            ).map_err(|_| GossipError::Inconsistent)?;
        }
        Ok(())
    }
}


// usage: gossip_storage().save_full_tree_head(&kt, &full_head);
impl GossipStorage {
    pub fn new() -> Self {
        Self {
            head: None,
        }
    }

    fn load_full_tree_head(&self, _kt: &KeyTransparency) -> Option<FullTreeHead> {
        self.head.clone()
    }

    fn save_full_tree_head(&mut self, _kt: &KeyTransparency, head: &FullTreeHead) {
        self.head = Some(head.clone());
    }
}

pub fn gossip_storage() -> std::sync::MutexGuard<'static, GossipStorage> {
    GLOBAL_GOSSIP_STORAGE.lock().expect("GLOBAL_GOSSIP_STORAGE lock")
}

impl Gossiper {
    pub fn new(
        storage: GossipStorage,
        kt: Option<KeyTransparency>
    ) -> Self {
        Self {
            storage,
            kt,
        }
    }

    pub fn append_gossip(
        &self,
        mut message: Vec<u8>, 
        origin_id: Vec<u8>,
    ) -> Vec<u8> {
        let kt = match &self.kt {
            Some(k) => k,
            None => return message,
        };

        let full = match self.storage.load_full_tree_head(kt) {
            Some(f) => f,
            None => return message,
        };

        let gossip = Gossip::new(full, origin_id, Vec::new());

        if let Ok(gossip_bytes) = gossip.encode() {
            message.extend(gossip_bytes);
        }
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libsignal_keytrans::{TreeHead, Signature, LastTreeHead};
    use std::time::SystemTime;

    fn create_test_full_tree_head() -> FullTreeHead {
        FullTreeHead {
            tree_head: Some(TreeHead {
                tree_size: 12345,
                timestamp: 1669123456789,
                signatures: vec![
                    Signature {
                        auditor_public_key: vec![0x01, 0x02, 0x03, 0x04],
                        signature: vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee],
                    },
                ],
            }),
            last: vec![
                vec![0x10, 0x20, 0x30, 0x40],
                vec![0x50, 0x60, 0x70, 0x80],
            ],
            distinguished: vec![
                vec![0xa1, 0xa2, 0xa3, 0xa4],
            ],
            full_auditor_tree_heads: vec![],
        }
    }

    #[test]
    fn test_encoding() {
        // Create test data
        let tree_head = create_test_full_tree_head();
        let origin_id = vec![0xde, 0xad, 0xbe, 0xef];
        let consistency_proof = vec![
            vec![0x11, 0x22, 0x33, 0x44],
            vec![0x55, 0x66, 0x77, 0x88],
            vec![0x99, 0xaa, 0xbb, 0xcc],
        ];

        let original_gossip = Gossip::new(tree_head, origin_id.clone(), consistency_proof.clone());
        let encoded = original_gossip.encode().expect("encoding should succeed");
        
        assert!(!encoded.is_empty(), "encoded data should not be empty");
        assert!(encoded.len() > 10, "encoded data should be substantial size");

        let decoded_gossip = Gossip::decode(&encoded).expect("decoding should succeed");

        assert_eq!(decoded_gossip.origin_id, origin_id, "origin_id should match");
        assert_eq!(
            decoded_gossip.consistency_proof.len(), 
            consistency_proof.len(), 
            "consistency_proof length should match"
        );
        for (i, (decoded, original)) in decoded_gossip.consistency_proof.iter()
            .zip(consistency_proof.iter()).enumerate() {
            assert_eq!(decoded, original, "consistency_proof element {} should match", i);
        }
        
        assert_eq!(
            decoded_gossip.full_tree_head.tree_head.as_ref().unwrap().tree_size,
            original_gossip.full_tree_head.tree_head.as_ref().unwrap().tree_size,
            "tree_size should match"
        );
        assert_eq!(
            decoded_gossip.full_tree_head.tree_head.as_ref().unwrap().timestamp,
            original_gossip.full_tree_head.tree_head.as_ref().unwrap().timestamp,
            "timestamp should match"
        );
        assert_eq!(
            decoded_gossip.full_tree_head.last.len(),
            original_gossip.full_tree_head.last.len(),
            "last length should match"
        );
        for (i, (decoded, original)) in decoded_gossip.full_tree_head.last.iter()
            .zip(original_gossip.full_tree_head.last.iter()).enumerate() {
            assert_eq!(decoded, original, "last element {} should match", i);
        }

        assert_eq!(
            decoded_gossip.full_tree_head.distinguished.len(),
            original_gossip.full_tree_head.distinguished.len(),
            "distinguished length should match"
        );
        for (i, (decoded, original)) in decoded_gossip.full_tree_head.distinguished.iter()
            .zip(original_gossip.full_tree_head.distinguished.iter()).enumerate() {
            assert_eq!(decoded, original, "distinguished element {} should match", i);
        }

        let re_encoded = decoded_gossip.encode().expect("re-encoding should succeed");
        assert_eq!(encoded, re_encoded, "round-trip encoding should produce identical results");
    }

    #[test]
    fn test_encoding_empty_data() {
        let full_tree_head = FullTreeHead {
            tree_head: Some(TreeHead {
                tree_size: 0,
                timestamp: 0,
                signatures: vec![],
            }),
            last: vec![],
            distinguished: vec![],
            full_auditor_tree_heads: vec![],
        };
        let gossip = Gossip::new(full_tree_head, vec![], vec![]);

        let encoded = gossip.encode().expect("encoding empty data should succeed");
        let decoded = Gossip::decode(&encoded).expect("decoding empty data should succeed");

        assert_eq!(decoded.origin_id.len(), 0, "empty origin_id should have length 0");
        assert_eq!(decoded.consistency_proof.len(), 0, "empty consistency_proof should have length 0");
        assert_eq!(
            decoded.full_tree_head.tree_head.as_ref().unwrap().tree_size,
            0,
            "empty tree_size should be 0"
        );
    }

    #[test]
    fn test_decode_invalid_data() {
        let invalid_data = vec![0xff, 0xff, 0xff, 0xff];
        let result = Gossip::decode(&invalid_data);
        assert!(result.is_err(), "decoding invalid data should fail");
        assert!(matches!(result.unwrap_err(), GossipError::Invalid));

        let empty_data = vec![];
        let result = Gossip::decode(&empty_data);
        assert!(result.is_err(), "decoding empty data should fail");
    }

    #[test]
    fn test_encoding_large_data() {
        let tree_head = create_test_full_tree_head();
        let origin_id = vec![0x42; 1000]; // 1KB of data
        let consistency_proof = vec![vec![0x33; 500]; 10]; // 10 proofs of 500 bytes each

        let gossip = Gossip::new(tree_head, origin_id.clone(), consistency_proof.clone());

        let encoded = gossip.encode().expect("encoding large data should succeed");
        let decoded = Gossip::decode(&encoded).expect("decoding large data should succeed");

        assert_eq!(decoded.origin_id, origin_id);
        assert_eq!(decoded.consistency_proof, consistency_proof);
    }
}
