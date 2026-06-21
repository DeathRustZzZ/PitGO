import type {
  ServiceResource,
  ServiceAppointment,
  FreeSlot,
  WorkingHours,
} from "../model/types";

export function computeFreeSlots(
  resources: ServiceResource[],
  appointments: ServiceAppointment[],
  date: Date,
  workingHours: WorkingHours,
  minSlotMin = 60,
): FreeSlot[] {
  const result: FreeSlot[] = [];

  for (const resource of resources) {
    const dayStart = new Date(date);
    dayStart.setHours(workingHours.startHour, 0, 0, 0);
    const dayEnd = new Date(date);
    dayEnd.setHours(workingHours.endHour, 0, 0, 0);

    const busy = appointments
      .filter((a) => a.resourceId === resource.id && a.status !== "cancelled")
      .map((a) => ({ start: new Date(a.startTime), end: new Date(a.endTime) }))
      .sort((a, b) => a.start.getTime() - b.start.getTime());

    let cursor = new Date(dayStart);

    for (const interval of busy) {
      if (interval.start.getTime() > cursor.getTime()) {
        const gapMs = interval.start.getTime() - cursor.getTime();
        if (gapMs >= minSlotMin * 60_000) {
          result.push({
            resourceId: resource.id,
            start: new Date(cursor),
            end: new Date(interval.start),
          });
        }
      }
      if (interval.end.getTime() > cursor.getTime()) {
        cursor = new Date(interval.end);
      }
    }

    if (cursor.getTime() < dayEnd.getTime()) {
      const gapMs = dayEnd.getTime() - cursor.getTime();
      if (gapMs >= minSlotMin * 60_000) {
        result.push({
          resourceId: resource.id,
          start: new Date(cursor),
          end: new Date(dayEnd),
        });
      }
    }
  }

  return result;
}
