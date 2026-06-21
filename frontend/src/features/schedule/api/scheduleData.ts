import type { ServiceResource, ServiceAppointment } from "../model/types";

export const mockResources: ServiceResource[] = [
  { id: "res1", name: "Пост 1", type: "post", color: "#3b82f6" },
  { id: "res2", name: "Пост 2", type: "post", color: "#8b5cf6" },
  { id: "res3", name: "Диагностика", type: "diagnostic", color: "#f59e0b" },
  { id: "res4", name: "Электрик", type: "electrical", color: "#10b981" },
];

const D = "2026-06-21";
function t(time: string): string {
  return `${D}T${time}:00`;
}

export const mockAppointments: ServiceAppointment[] = [
  // Пост 1
  {
    id: "apt1",
    resourceId: "res1",
    orderId: "o1",
    clientName: "Иванов А.П.",
    carLabel: "Toyota Camry А001АА",
    description: "ТО-1: масло, фильтры",
    startTime: t("09:00"),
    endTime: t("11:00"),
    status: "completed",
  },
  {
    id: "apt2",
    resourceId: "res1",
    orderId: "o5",
    clientName: "Новиков С.А.",
    carLabel: "VW Polo Р555РР",
    description: "Плановое ТО",
    startTime: t("13:00"),
    endTime: t("15:30"),
    status: "scheduled",
  },
  // Пост 2
  {
    id: "apt3",
    resourceId: "res2",
    orderId: "o2",
    clientName: "Петрова М.С.",
    carLabel: "Kia Rio С222СС",
    description: "Тормозные колодки",
    startTime: t("10:00"),
    endTime: t("12:30"),
    status: "in_progress",
  },
  {
    id: "apt4",
    resourceId: "res2",
    orderId: "o4",
    clientName: "Козлова Е.В.",
    carLabel: "Hyundai Tucson М444ММ",
    description: "Стойка стабилизатора",
    startTime: t("14:00"),
    endTime: t("16:00"),
    status: "scheduled",
  },
  // Диагностика
  {
    id: "apt5",
    resourceId: "res3",
    orderId: "o3",
    clientName: "Сидоров Д.Н.",
    carLabel: "Lada Vesta Е333ЕЕ",
    description: "Компьютерная диагностика",
    startTime: t("11:30"),
    endTime: t("13:00"),
    status: "in_progress",
  },
  {
    id: "apt6",
    resourceId: "res3",
    clientName: "Смирнов В.А.",
    carLabel: "Ford Focus Х789ОА",
    description: "Проверка электрики",
    startTime: t("15:30"),
    endTime: t("16:30"),
    status: "scheduled",
  },
  // Электрик
  {
    id: "apt7",
    resourceId: "res4",
    clientName: "Иванов А.П.",
    carLabel: "BMW X5 В123ВВ",
    description: "Замена генератора",
    startTime: t("09:00"),
    endTime: t("11:30"),
    status: "in_progress",
  },
  {
    id: "apt8",
    resourceId: "res4",
    clientName: "Фёдоров П.И.",
    carLabel: "Renault Logan К456МС",
    description: "Диагностика АКБ",
    startTime: t("14:30"),
    endTime: t("16:00"),
    status: "scheduled",
  },
];
