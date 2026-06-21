import { useQuery } from "@tanstack/react-query";
import { scheduleApi } from "../api/scheduleApi";
import { ScheduleCalendar } from "./ScheduleCalendar";

export function ScheduleWidget({ date }: { date: Date }) {
  const start = new Date(date);
  start.setHours(0, 0, 0, 0);
  const end = new Date(date);
  end.setHours(23, 59, 59, 999);

  const { data: resources = [] } = useQuery({
    queryKey: ["schedule-resources"],
    queryFn: scheduleApi.listResources,
    staleTime: Infinity,
  });

  const { data: appointments = [] } = useQuery({
    queryKey: ["schedule-appointments", start.toDateString()],
    queryFn: () =>
      scheduleApi.listAppointments({
        from: start.toISOString(),
        to: end.toISOString(),
      }),
  });

  return (
    <ScheduleCalendar
      date={date}
      resources={resources}
      appointments={appointments}
    />
  );
}
