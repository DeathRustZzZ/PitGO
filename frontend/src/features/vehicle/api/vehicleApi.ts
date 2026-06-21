import { ApiError } from "../../../shared/api/client";
import type { Vehicle, VehicleCreateInput } from "./types";

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

const mockVehicles: Vehicle[] = [
  {
    id: "v1",
    clientId: "demo",
    make: "Toyota",
    model: "Camry",
    year: 2020,
    plateNumber: "А001АА 77",
    vin: "XTA210990B2345678",
    mileage: 85000,
    status: "active",
    createdAt: "2024-01-15T10:00:00Z",
  },
  {
    id: "v2",
    clientId: "demo",
    make: "BMW",
    model: "X5",
    year: 2018,
    plateNumber: "В123ВВ 77",
    mileage: 142000,
    status: "active",
    createdAt: "2024-03-20T14:30:00Z",
  },
];

export async function getVehicles(clientId: string): Promise<Vehicle[]> {
  await delay(400);
  return mockVehicles.filter((v) => v.clientId === clientId);
}

export async function createVehicle(
  clientId: string,
  input: VehicleCreateInput,
): Promise<Vehicle> {
  await delay(600);

  const existing = mockVehicles.find(
    (v) => v.plateNumber.toLowerCase() === input.plateNumber.toLowerCase(),
  );
  if (existing) {
    throw new ApiError("Автомобиль с таким номером уже добавлен", 409);
  }

  const vehicle: Vehicle = {
    id: crypto.randomUUID(),
    clientId,
    ...input,
    status: "active",
    createdAt: new Date().toISOString(),
  };
  mockVehicles.push(vehicle);
  return vehicle;
}
