export type Client = {
  id: string;
  fullName: string;
  phone: string;
  email?: string;
  comment?: string;
  createdAt: string;
};

export type Car = {
  id: string;
  clientId: string;
  vin?: string;
  plateNumber: string;
  make: string;
  model: string;
  year: number;
  mileage?: number;
  comment?: string;
};

export type OrderStatus =
  | "new"
  | "diagnostics"
  | "waiting_parts"
  | "in_progress"
  | "ready"
  | "completed"
  | "cancelled";

export type OrderWork = {
  id: string;
  name: string;
  price: number;
  quantity: number;
};

export type OrderPart = {
  id: string;
  name: string;
  sku?: string;
  price: number;
  quantity: number;
};

export type Order = {
  id: string;
  orderNumber: string;
  clientId: string;
  carId: string;
  status: OrderStatus;
  description: string;
  works: OrderWork[];
  parts: OrderPart[];
  totalPrice: number;
  createdAt: string;
  completedAt?: string;
};

export type InventoryItem = {
  id: string;
  name: string;
  sku: string;
  quantity: number;
  minQuantity: number;
  purchasePrice: number;
  salePrice: number;
  supplierName?: string;
};

export type ReminderType = "service" | "oil_change" | "inspection" | "custom";
export type ReminderStatus = "planned" | "sent" | "cancelled";

export type Reminder = {
  id: string;
  clientId: string;
  carId?: string;
  type: ReminderType;
  dueDate: string;
  text: string;
  status: ReminderStatus;
};
