import { useMemo } from "react";
import { useNavigate } from "react-router-dom";
import type {
  ServiceResource,
  ServiceAppointment,
  FreeSlot,
} from "../model/types";
import { computeFreeSlots } from "../lib/availability";

const WORK_START = 8;
const WORK_END = 20;
const PX_PER_HOUR = 56;
const PX_PER_MIN = PX_PER_HOUR / 60;
const TOTAL_HOURS = WORK_END - WORK_START;
const GRID_H = TOTAL_HOURS * PX_PER_HOUR;

const HOURS = Array.from({ length: TOTAL_HOURS }, (_, i) => WORK_START + i);

function hexToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function toMin(isoStr: string): number {
  const d = new Date(isoStr);
  return (d.getHours() - WORK_START) * 60 + d.getMinutes();
}

function AppointmentBlock({
  apt,
  color,
  onNavigate,
}: {
  apt: ServiceAppointment;
  color: string;
  onNavigate?: () => void;
}) {
  const top = Math.max(0, toMin(apt.startTime) * PX_PER_MIN);
  const dur =
    (new Date(apt.endTime).getTime() - new Date(apt.startTime).getTime()) /
    60_000;
  const height = Math.max(Math.min(dur * PX_PER_MIN, GRID_H - top), 22);

  return (
    <div
      onClick={onNavigate}
      title={`${apt.clientName} · ${apt.carLabel}\n${apt.description}`}
      style={{
        position: "absolute",
        top,
        left: 2,
        right: 2,
        height,
        background: hexToRgba(color, 0.15),
        borderLeft: `3px solid ${color}`,
        borderRadius: 4,
        padding: "3px 6px",
        overflow: "hidden",
        cursor: onNavigate ? "pointer" : "default",
        userSelect: "none",
        zIndex: 1,
      }}
    >
      <p
        style={{
          fontSize: 11,
          fontWeight: 600,
          color,
          lineHeight: 1.3,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
          margin: 0,
        }}
      >
        {apt.clientName}
      </p>
      {height > 34 && (
        <p
          style={{
            fontSize: 10,
            color: "var(--color-hint)",
            lineHeight: 1.2,
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
            margin: 0,
          }}
        >
          {apt.carLabel}
        </p>
      )}
    </div>
  );
}

function FreeSlotBlock({ slot, color }: { slot: FreeSlot; color: string }) {
  const startMin =
    (slot.start.getHours() - WORK_START) * 60 + slot.start.getMinutes();
  const dur = (slot.end.getTime() - slot.start.getTime()) / 60_000;
  const top = startMin * PX_PER_MIN;
  const height = dur * PX_PER_MIN;

  if (height < 18) return null;

  return (
    <div
      style={{
        position: "absolute",
        top: top + 2,
        left: 4,
        right: 4,
        height: height - 4,
        border: `1px dashed ${hexToRgba(color, 0.28)}`,
        borderRadius: 4,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        pointerEvents: "none",
      }}
    >
      {height >= 42 && (
        <span
          style={{
            fontSize: 10,
            color: "var(--color-hint)",
            userSelect: "none",
          }}
        >
          Свободно
        </span>
      )}
    </div>
  );
}

type Props = {
  date: Date;
  resources: ServiceResource[];
  appointments: ServiceAppointment[];
};

export function ScheduleCalendar({ date, resources, appointments }: Props) {
  const navigate = useNavigate();

  const freeSlots = useMemo(
    () =>
      computeFreeSlots(resources, appointments, date, {
        startHour: WORK_START,
        endHour: WORK_END,
      }),
    [resources, appointments, date],
  );

  const aptByResource = useMemo(() => {
    const map: Record<string, ServiceAppointment[]> = {};
    for (const apt of appointments) {
      if (!map[apt.resourceId]) map[apt.resourceId] = [];
      map[apt.resourceId].push(apt);
    }
    return map;
  }, [appointments]);

  const slotsByResource = useMemo(() => {
    const map: Record<string, FreeSlot[]> = {};
    for (const slot of freeSlots) {
      if (!map[slot.resourceId]) map[slot.resourceId] = [];
      map[slot.resourceId].push(slot);
    }
    return map;
  }, [freeSlots]);

  const colTemplate = `52px repeat(${resources.length}, minmax(0, 1fr))`;

  return (
    <div style={{ overflowX: "auto" }}>
      {/* Шапка ресурсов */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: colTemplate,
          borderBottom: "1px solid var(--color-border)",
          background: "var(--color-bg)",
          position: "sticky",
          top: 0,
          zIndex: 2,
        }}
      >
        <div style={{ borderRight: "1px solid var(--color-border)" }} />
        {resources.map((r) => (
          <div
            key={r.id}
            style={{
              padding: "6px 8px",
              borderLeft: "1px solid var(--color-border)",
              display: "flex",
              alignItems: "center",
              gap: 6,
            }}
          >
            <div
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: r.color,
                flexShrink: 0,
              }}
            />
            <span
              style={{
                fontSize: 12,
                fontWeight: 600,
                color: "var(--color-text)",
                whiteSpace: "nowrap",
              }}
            >
              {r.name}
            </span>
          </div>
        ))}
      </div>

      {/* Тело сетки */}
      <div style={{ overflowY: "auto", maxHeight: 420 }}>
        <div
          style={{
            display: "grid",
            gridTemplateColumns: colTemplate,
            height: GRID_H,
          }}
        >
          {/* Ось времени */}
          <div
            style={{
              position: "relative",
              borderRight: "1px solid var(--color-border)",
            }}
          >
            {HOURS.map((h) => (
              <div
                key={h}
                style={{
                  position: "absolute",
                  top: (h - WORK_START) * PX_PER_HOUR - 7,
                  right: 4,
                  fontSize: 10,
                  color: "var(--color-hint)",
                  lineHeight: 1,
                  userSelect: "none",
                }}
              >
                {String(h).padStart(2, "0")}:00
              </div>
            ))}
          </div>

          {/* Колонки ресурсов */}
          {resources.map((r) => (
            <div
              key={r.id}
              style={{
                position: "relative",
                borderLeft: "1px solid var(--color-border)",
              }}
            >
              {/* Линии часов */}
              {HOURS.map((h) => (
                <div
                  key={h}
                  style={{
                    position: "absolute",
                    top: (h - WORK_START) * PX_PER_HOUR,
                    left: 0,
                    right: 0,
                    height: 1,
                    background: "var(--color-border)",
                    opacity: 0.5,
                  }}
                />
              ))}
              {/* Свободные окна */}
              {slotsByResource[r.id]?.map((slot, i) => (
                <FreeSlotBlock key={i} slot={slot} color={r.color} />
              ))}
              {/* Записи */}
              {aptByResource[r.id]?.map((apt) => (
                <AppointmentBlock
                  key={apt.id}
                  apt={apt}
                  color={r.color}
                  onNavigate={
                    apt.orderId
                      ? () => navigate(`/app/orders/${apt.orderId}`)
                      : undefined
                  }
                />
              ))}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
