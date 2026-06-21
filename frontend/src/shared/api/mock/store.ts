import type {
  Client,
  Car,
  Order,
  InventoryItem,
  Reminder,
  OrderStatus,
} from "@/shared/types";
import {
  mockClients,
  mockCars,
  mockOrders,
  mockInventory,
  mockReminders,
} from "./data";

const delay = (ms = 200) => new Promise((resolve) => setTimeout(resolve, ms));

let clients = [...mockClients];
let cars = [...mockCars];
let orders = [...mockOrders];
let inventory = [...mockInventory];
let reminders = [...mockReminders];

let nextId = 100;
const newId = () => String(++nextId);

// --- Clients ---

export const clientsApi = {
  async list(): Promise<Client[]> {
    await delay();
    return [...clients];
  },

  async get(id: string): Promise<Client | undefined> {
    await delay();
    return clients.find((c) => c.id === id);
  },

  async create(data: Omit<Client, "id" | "createdAt">): Promise<Client> {
    await delay();
    const client: Client = {
      ...data,
      id: newId(),
      createdAt: new Date().toISOString(),
    };
    clients = [...clients, client];
    return client;
  },

  async update(
    id: string,
    data: Partial<Omit<Client, "id" | "createdAt">>,
  ): Promise<Client> {
    await delay();
    clients = clients.map((c) => (c.id === id ? { ...c, ...data } : c));
    return clients.find((c) => c.id === id)!;
  },

  async delete(id: string): Promise<void> {
    await delay();
    clients = clients.filter((c) => c.id !== id);
  },
};

// --- Cars ---

export const carsApi = {
  async list(): Promise<Car[]> {
    await delay();
    return [...cars];
  },

  async listByClient(clientId: string): Promise<Car[]> {
    await delay();
    return cars.filter((c) => c.clientId === clientId);
  },

  async get(id: string): Promise<Car | undefined> {
    await delay();
    return cars.find((c) => c.id === id);
  },

  async create(data: Omit<Car, "id">): Promise<Car> {
    await delay();
    const car: Car = { ...data, id: newId() };
    cars = [...cars, car];
    return car;
  },

  async update(id: string, data: Partial<Omit<Car, "id">>): Promise<Car> {
    await delay();
    cars = cars.map((c) => (c.id === id ? { ...c, ...data } : c));
    return cars.find((c) => c.id === id)!;
  },

  async delete(id: string): Promise<void> {
    await delay();
    cars = cars.filter((c) => c.id !== id);
  },
};

// --- Orders ---

export const ordersApi = {
  async list(): Promise<Order[]> {
    await delay();
    return [...orders];
  },

  async get(id: string): Promise<Order | undefined> {
    await delay();
    return orders.find((o) => o.id === id);
  },

  async listByClient(clientId: string): Promise<Order[]> {
    await delay();
    return orders.filter((o) => o.clientId === clientId);
  },

  async listByCar(carId: string): Promise<Order[]> {
    await delay();
    return orders.filter((o) => o.carId === carId);
  },

  async create(
    data: Omit<Order, "id" | "orderNumber" | "createdAt" | "totalPrice">,
  ): Promise<Order> {
    await delay();
    const num = orders.length + 1;
    const order: Order = {
      ...data,
      id: newId(),
      orderNumber: `ЗН-2026-${String(num).padStart(3, "0")}`,
      totalPrice: 0,
      createdAt: new Date().toISOString(),
    };
    orders = [...orders, order];
    return order;
  },

  async update(
    id: string,
    data: Partial<Omit<Order, "id" | "orderNumber" | "createdAt">>,
  ): Promise<Order> {
    await delay();
    orders = orders.map((o) => {
      if (o.id !== id) return o;
      const updated = { ...o, ...data };
      const worksTotal = updated.works.reduce(
        (s, w) => s + w.price * w.quantity,
        0,
      );
      const partsTotal = updated.parts.reduce(
        (s, p) => s + p.price * p.quantity,
        0,
      );
      updated.totalPrice = worksTotal + partsTotal;
      return updated;
    });
    return orders.find((o) => o.id === id)!;
  },

  async updateStatus(id: string, status: OrderStatus): Promise<Order> {
    await delay();
    orders = orders.map((o) => {
      if (o.id !== id) return o;
      const completedAt =
        status === "completed" ? new Date().toISOString() : o.completedAt;
      return { ...o, status, completedAt };
    });
    return orders.find((o) => o.id === id)!;
  },

  async delete(id: string): Promise<void> {
    await delay();
    orders = orders.filter((o) => o.id !== id);
  },
};

// --- Inventory ---

export const inventoryApi = {
  async list(): Promise<InventoryItem[]> {
    await delay();
    return [...inventory];
  },

  async get(id: string): Promise<InventoryItem | undefined> {
    await delay();
    return inventory.find((i) => i.id === id);
  },

  async create(data: Omit<InventoryItem, "id">): Promise<InventoryItem> {
    await delay();
    const item: InventoryItem = { ...data, id: newId() };
    inventory = [...inventory, item];
    return item;
  },

  async update(
    id: string,
    data: Partial<Omit<InventoryItem, "id">>,
  ): Promise<InventoryItem> {
    await delay();
    inventory = inventory.map((i) => (i.id === id ? { ...i, ...data } : i));
    return inventory.find((i) => i.id === id)!;
  },

  async delete(id: string): Promise<void> {
    await delay();
    inventory = inventory.filter((i) => i.id !== id);
  },
};

// --- Reminders ---

export const remindersApi = {
  async list(): Promise<Reminder[]> {
    await delay();
    return [...reminders];
  },

  async get(id: string): Promise<Reminder | undefined> {
    await delay();
    return reminders.find((r) => r.id === id);
  },

  async create(data: Omit<Reminder, "id">): Promise<Reminder> {
    await delay();
    const reminder: Reminder = { ...data, id: newId() };
    reminders = [...reminders, reminder];
    return reminder;
  },

  async update(
    id: string,
    data: Partial<Omit<Reminder, "id">>,
  ): Promise<Reminder> {
    await delay();
    reminders = reminders.map((r) => (r.id === id ? { ...r, ...data } : r));
    return reminders.find((r) => r.id === id)!;
  },

  async delete(id: string): Promise<void> {
    await delay();
    reminders = reminders.filter((r) => r.id !== id);
  },
};
