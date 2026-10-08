//! A repo's documents and specs: read, search, link resolution and the guarded write. Pure
//! functions and wire types with no web framework, so the daemon and the gateway share them
//! (br-5e4k).

pub mod documents;
pub mod specs;
