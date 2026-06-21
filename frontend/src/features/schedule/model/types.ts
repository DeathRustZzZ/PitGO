export type AppointmentStatus =
  | "scheduled"
  | "in_progress"
  | "completed"
  | "cancelled";

export type ResourceType = "post" | "diagnostic" | "electrical" | "mechanic";

export type ServiceResource = {
  id: string;
  name: string;
  type: ResourceType;
  color: string;
};

export type ServiceAppointment = {
  id: string;
  resourceId: string;
  orderId?: string;
  clientName: string;
  carLabel: string;
  description: string;
  startTime: string;
  endTime: string;
  status: AppointmentStatus;
};

export type WorkingHours = {
  startHour: number;
  endHour: number;
};

export type FreeSlot = {
  resourceId: string;
  start: Date;
  end: Date;
};
