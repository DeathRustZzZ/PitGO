pub mod aggregate;
pub mod error;
pub mod event;
pub mod snapshot;
pub mod state;

pub use aggregate::VehicleOwnership;
pub use error::OwnershipError;
pub use event::VehicleOwnershipEvent;
pub use snapshot::OwnershipEligibilitySnapshot;
pub use state::{OwnershipStatus, OwnershipType};
