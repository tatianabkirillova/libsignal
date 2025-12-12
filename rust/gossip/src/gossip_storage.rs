/*
Gossip Service

This service should 
1. periodically call kt.verify_monitor() 
2. check incoming gossip's consistency using kt.verify_distinguished() and update last_distinguished_tree_head if OK

FullTreeHead = tree head incoming from gossip to verify
last_tree_head = current trusted head (updated after valid gossip responses)
last_distinguished_tree_head = last head directly signed by server (never updated by gossip)
*/

use libsignal_keytrans::{
    LastTreeHead
};

#[derive(Clone, Debug)]
pub struct KtState {
    last_tree_head: Option<LastTreeHead>, // (TreeHead, TreeRoot)
    last_distinguished_tree_head: Option<LastTreeHead>,
}

impl KtState {
    pub fn empty() -> Self {
        KtState {
            last_tree_head: None,
            last_distinguished_tree_head: None,
        }
    }

    pub fn last_tree_head(&self) -> Option<&LastTreeHead> {
        self.last_tree_head.as_ref()
    }

    pub fn last_distinguished_tree_head(&self) -> Option<&LastTreeHead> {
        self.last_distinguished_tree_head.as_ref()
    }

    pub fn set_last_tree_head(&mut self, tree_head: LastTreeHead) {
        self.last_tree_head = Some(tree_head);
    }

    pub fn set_last_distinguished_tree_head(&mut self, tree_head: LastTreeHead) {
        self.last_distinguished_tree_head = Some(tree_head);
    }

    pub fn has_tree_head(&self) -> bool {
        self.last_tree_head.is_some()
    }

    pub fn has_distinguished_tree_head(&self) -> bool {
        self.last_distinguished_tree_head.is_some()
    }

    pub fn is_initialized(&self) -> bool {
        self.has_tree_head() && self.has_distinguished_tree_head()
    }
}

pub fn save_state(_state: &KtState) {
    // TODO: save somewhere
}

pub fn load_state() -> KtState {
    // TODO: read from from that somewhere
    KtState::empty()
}
