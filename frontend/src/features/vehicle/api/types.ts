// Контракт данных Vehicle на стороне фронта.
// Согласуется с доменом backend (см. docs/domain/vehicle).

export type VehicleStatus = "active" | "archived";

export interface VehicleCreateInput {
  make: string;
  model: string;
  year: number;
  plateNumber: string;
  vin?: string;
  mileage?: number;
}

export interface Vehicle {
  id: string;
  clientId: string;
  make: string;
  model: string;
  year: number;
  plateNumber: string;
  vin?: string;
  mileage?: number;
  status: VehicleStatus;
  createdAt: string;
}
