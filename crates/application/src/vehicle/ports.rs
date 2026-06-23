use crate::error::RepositoryError;
use domain::VehicleId;
use domain::vehicle::aggregate::Vehicle;

pub trait VehicleRepository {
    fn save(&self, vehicle: &Vehicle) -> Result<(), RepositoryError>;

    fn find_by_id(&self, vehicle_id: VehicleId) -> Result<Option<Vehicle>, RepositoryError>;
}
