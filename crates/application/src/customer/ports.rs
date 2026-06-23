use crate::error::RepositoryError;
use domain::CustomerId;
use domain::VehicleId;
use domain::customer::aggregate::Customer;
use domain::vehicle::aggregate::Vehicle;

pub trait CustomerRepository {
    fn save(&self, customer: &mut Customer) -> Result<(), RepositoryError>;

    fn find_by_id(&self, customer_id: CustomerId) -> Result<Option<Customer>, RepositoryError>;
}

pub trait VehicleRepository {
    fn save(&self, vehicle: &mut Vehicle) -> Result<(), RepositoryError>;

    fn find_by_id(&self, vehicle_id: VehicleId) -> Result<Option<Vehicle>, RepositoryError>;
}

pub trait VehicleOwnershipRepository {}
