// use libsignal_keytrans::{
//     FullTreeHead, LastTreeHead, KeyTransparency
// };
// use prost::Message;
// use once_cell::sync::Lazy;
// use std::{sync::Mutex};
// use std::time::SystemTime;

mod gossip;
pub use gossip::*;

mod gossip_storage;
pub use gossip_storage::*;

pub mod gossip_service;
pub use gossip_service::*;

// pub mod proto {
//     tonic::include_proto!("gossip");
// }

// pub struct Gossiper {
//     storage: GossipStorage,
//     kt: Option<KeyTransparency>,
// }

// pub struct GossipStorage {
//     head: Option<FullTreeHead>,
// }

// static GLOBAL_GOSSIP_STORAGE: Lazy<Mutex<GossipStorage>> = Lazy::new(|| {
//     Mutex::new(GossipStorage::new())
// });

// static GLOBAL_GOSSIPER: Lazy<Mutex<Option<Gossiper>>> = Lazy::new(|| {
//     Mutex::new(None)
// });

// pub fn init_gossiper(kt: KeyTransparency) {
//     let mut guard = GLOBAL_GOSSIPER
//         .lock()
//         .expect("GLOBAL_GOSSIPER lock");
//     if guard.is_none() {
//         let storage = GossipStorage::new();
//         *guard = Some(Gossiper::new(storage, kt));
//     }
// }

// /*
// Usage:
// init_gossiper(kt_config);
// if let Some(g) = gossiper().as_ref() {
//     let msg_with_gossip = g.append_gossip(message_bytes, protocol_address);
// }
// */
// pub fn gossiper() -> std::sync::MutexGuard<'static, Option<Gossiper>> {
//     GLOBAL_GOSSIPER
//         .lock()
//         .expect("GLOBAL_GOSSIPER lock")
// }

// // usage: gossip_storage().save_full_tree_head(&kt, &full_head);
// impl GossipStorage {
//     pub fn new() -> Self {
//         Self {
//             head: None,
//         }
//     }

//     pub fn load_full_tree_head(&self, _kt: &KeyTransparency) -> Option<FullTreeHead> {
//         self.head.clone()
//     }

//     pub fn save_full_tree_head(&mut self, _kt: &KeyTransparency, head: &FullTreeHead) {
//         self.head = Some(head.clone());
//     }
// }

// pub fn gossip_storage() -> std::sync::MutexGuard<'static, GossipStorage> {
//     GLOBAL_GOSSIP_STORAGE.lock().expect("GLOBAL_GOSSIP_STORAGE lock")
// }

// impl Gossiper {
//     pub fn new(
//         storage: GossipStorage,
//         kt: KeyTransparency
//     ) -> Self {
//         Self {
//             storage,
//             kt: Some(kt),
//         }
//     }

//     pub fn build_gossip(
//         &self,
//         protocol_address: Vec<u8>,
//         now: SystemTime,
//     ) -> Option<Gossip> {
//         let kt = self.kt.as_ref()?;
//         let full = self.storage.load_full_tree_head(kt)?;
        
//         let gossip = Gossip::new(full, protocol_address, Vec::new(), now);
//         Some(gossip)
//     }
// }
