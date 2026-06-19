pub mod aggregate;
pub mod error;
pub mod event;
pub mod permit;
pub mod state;

pub use aggregate::Customer;
pub use error::CustomerError;
pub use event::CustomerEvent;
pub use permit::ActivationPermit;
pub use state::CustomerStatus;
