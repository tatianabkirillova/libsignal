use libsignal_keytrans::{
    FullTreeHead, LastTreeHead, KeyTransparency, TreeRoot
};

use prost::Message;

use std::time::SystemTime;
pub mod proto {
    tonic::include_proto!("gossip");
}

#[derive(Debug, Clone, PartialEq)]
pub enum GossipError {
    Invalid,
    Inconsistent
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gossip {
    pub full_tree_head : FullTreeHead,  // consists consistency proof 
    pub tree_root: TreeRoot,
    pub timestamp: SystemTime,
}

impl Gossip {
    pub fn new(
        full_tree_head: FullTreeHead,
        tree_root: TreeRoot,
        timestamp: SystemTime,
    ) -> Self {
        Self {
            full_tree_head,
            tree_root,
            timestamp,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, GossipError> {
        let mut th_bytes = Vec::new();
        self.full_tree_head
            .encode(&mut th_bytes)
            .map_err(|_| GossipError::Invalid)?;

        let proto = proto::Gossip {
            full_tree_head: th_bytes,
            tree_root: self.tree_root.to_vec(),
            timestamp: self.timestamp.duration_since(SystemTime::UNIX_EPOCH).map_err(|_| GossipError::Invalid)?.as_millis() as i64,
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

        let proto_root = proto.tree_root.as_slice(); 

        if proto_root.len() != 32 {
            return Err(GossipError::Invalid);
        }

        let mut tree_root = [0u8; 32];
        tree_root.copy_from_slice(proto_root);

        Ok(Self {
            full_tree_head,
            tree_root,
            timestamp: SystemTime::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libsignal_keytrans::{TreeHead, Signature, TreeRoot};

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

    fn create_test_tree_root() -> TreeRoot {
        [0xAA; 32]
    }

    #[test]
    fn test_encoding() {
        // Create test data
        let tree_head = create_test_full_tree_head();
        let tree_root = create_test_tree_root();
        let timestamp = SystemTime::now();

        let original_gossip = Gossip::new(tree_head, tree_root, timestamp);
        let encoded = original_gossip.encode().expect("encoding should succeed");
        
        assert!(!encoded.is_empty(), "encoded data should not be empty");
        assert!(encoded.len() > 10, "encoded data should be substantial size");

        let decoded_gossip = Gossip::decode(&encoded).expect("decoding should succeed");

        assert_eq!(decoded_gossip.full_tree_head, original_gossip.full_tree_head, "full_tree_head should match");
        assert_eq!(decoded_gossip.tree_root, original_gossip.tree_root, "tree_root should match");
    }
}