/**
 * We assume a mixnet protocol in which each user can populate
 * a mailbox information to be asynchronously reachable
 *
 * In this simple implementation, the mailbox would a node from one of the first or second layer
 *
 * As soon as we have a real and sound mixnet deployment, mailbox information should
 * be provided according to the choices they make.
 */
use rand::seq::SliceRandom;

use crate::mixnodes::mixnode::Mixnode;

#[derive(Clone, Debug)]
pub struct MailBox {
    pub mixid: u32,
    pub is_malicious: bool,
}

impl MailBox {
    /// Construct a mailbox by choosing randomly a
    /// mixnode's mixid from one of the provided mixnet layers
    pub fn new(from_layers: &[Vec<Mixnode>]) -> MailBox {
        let mut rng = rand::thread_rng();
        // if the unwrap fail, it is a bug
        let layer = from_layers.choose(&mut rng).unwrap();
        let mixnode = layer.choose(&mut rng).unwrap();

        MailBox {
            mixid: mixnode.mixid,
            is_malicious: mixnode.is_malicious,
        }
    }
}
