use libsignal_keytrans::{KeyTransparency, LastTreeHead, FullTreeHead, TreeHead, TreeRoot, Signature};
use crate::gossip::Gossip;     
use crate::gossip::GossipError;
use std::time::SystemTime;

use crate::gossip_storage::{KtState, load_state, save_state};

pub struct GossipService {
    kt: KeyTransparency,
    state: KtState,
}

pub mod gossip_test {
    use super::*;
    pub fn create_test_full_tree_head() -> FullTreeHead {
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

    pub fn create_test_tree_root() -> TreeRoot {
        [0xAA; 32]
    }

    pub fn create_test_gossip() -> Gossip {
        Gossip {
            full_tree_head: create_test_full_tree_head(),
            tree_root: create_test_tree_root(),
            timestamp: SystemTime::now(),
        }
    }
}

impl GossipService {
    pub fn new(kt: KeyTransparency) -> Self {
        let state = load_state();
        Self { kt, state }
    }

    pub fn state(&self) -> &KtState {
        &self.state
    }

    pub fn process_incoming_gossip(&mut self, bytes: &[u8]) -> Result<(), GossipError> {
        let gossip = Gossip::decode(bytes)?;
        let local_last = self.state.last_tree_head();
        let local_distinguished = self.state.last_distinguished_tree_head();

        if let Some(ld) = local_distinguished {
            self.kt
                .verify_distinguished(&gossip.full_tree_head, local_last, ld)
                .map_err(|_| GossipError::Invalid)?;
        } else {
            return Err(GossipError::Invalid);
        }

        if let Some(tree_head) = gossip.full_tree_head.tree_head.clone() {
            self.state.set_last_tree_head((tree_head, gossip.tree_root));
            save_state(&self.state);
        }

        Ok(())
    }

    pub fn run_monitor_once(
        &mut self,
        req: &libsignal_keytrans::MonitorRequest,
        resp: &libsignal_keytrans::MonitorResponse,
        ctx: libsignal_keytrans::MonitorContext,
        now: SystemTime,
    ) -> Result<(), libsignal_keytrans::Error> {
        let update = self.kt.verify_monitor(req, resp, ctx, now)?;

        let new_last: LastTreeHead = (update.tree_head, update.tree_root);

        self.state.set_last_tree_head(new_last.clone());
        self.state.set_last_distinguished_tree_head(new_last);
        save_state(&self.state);

        Ok(())
    }
}
