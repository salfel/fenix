mod l1;
mod l2;
mod setup;

pub use l2::{register_page, unregister_page, L2SmallPageTableEntry};
pub use setup::initialize;
