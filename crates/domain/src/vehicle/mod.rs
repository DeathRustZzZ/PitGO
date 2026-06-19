pub mod aggregate;
pub mod error;
pub mod event;
pub mod permit;
pub mod state;

pub use aggregate::Vehicle;
pub use error::VehicleError;
pub use event::VehicleEvent;
pub use permit::VehicleActivationPermit;
pub use state::VehicleStatus;
