use domain::CustomerId;
use domain::VehicleId;

pub trait CustomerRepository {
    fn save(&self, customer: &CustomerId) -> Result<()>;
    fn find_by_id(&self, customer_id: CustomerId) -> Result<Option<Customer>, RepositoryError>;
}

pub trait VehicleRepository {
    // подумай: что нужно для create + activate vehicle?
    fn save(&self, vehicle: &VehicleId) -> Result<(), RepositoryError>;
}

pub trait VehicleOwnershipRepository {
    // подумай: что нужно для start + verify ownership?
}
