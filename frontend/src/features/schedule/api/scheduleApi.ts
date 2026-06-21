import { mockResources, mockAppointments } from "./scheduleData";
import type { ServiceResource, ServiceAppointment } from "../model/types";

const delay = (ms = 200) => new Promise<void>((r) => setTimeout(r, ms));

export const scheduleApi = {
  async listResources(): Promise<ServiceResource[]> {
    await delay();
    return [...mockResources];
  },

  async listAppointments({
    from,
    to,
  }: {
    from: string;
    to: string;
  }): Promise<ServiceAppointment[]> {
    await delay();
    const fromMs = new Date(from).getTime();
    const toMs = new Date(to).getTime();
    return mockAppointments.filter((a) => {
      const startMs = new Date(a.startTime).getTime();
      return startMs >= fromMs && startMs <= toMs;
    });
  },
};
