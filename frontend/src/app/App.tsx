import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

// Старые публичные страницы (landing, регистрация, гараж)
import { LandingPage } from "../features/landing/LandingPage";
import { CustomerRegistrationPage } from "../features/customer/pages/CustomerRegistrationPage";
import { VehicleGaragePage } from "../features/vehicle/pages/VehicleGaragePage";
import { VehicleCreatePage } from "../features/vehicle/pages/VehicleCreatePage";

// CRM-раздел (layout + все страницы управления)
import { AppLayout } from "./layout/AppLayout";
import { DashboardPage } from "@/pages/dashboard/DashboardPage";
import { OrdersPage } from "@/pages/orders/OrdersPage";
import { OrderDetailPage } from "@/pages/orders/OrderDetailPage";
import { OrderCreatePage } from "@/pages/orders/OrderCreatePage";
import { OrderEditPage } from "@/pages/orders/OrderEditPage";
import { ClientsPage } from "@/pages/clients/ClientsPage";
import { ClientDetailPage } from "@/pages/clients/ClientDetailPage";
import { CarsPage } from "@/pages/cars/CarsPage";
import { CarDetailPage } from "@/pages/cars/CarDetailPage";
import { InventoryPage } from "@/pages/inventory/InventoryPage";
import { RemindersPage } from "@/pages/reminders/RemindersPage";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { staleTime: 30_000, retry: false },
  },
});

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          {/* Публичные страницы */}
          <Route path="/" element={<LandingPage />} />
          <Route path="/register" element={<CustomerRegistrationPage />} />
          <Route path="/vehicles" element={<VehicleGaragePage />} />
          <Route path="/vehicles/new" element={<VehicleCreatePage />} />

          {/* CRM — под отдельным префиксом /app */}
          <Route path="/app" element={<AppLayout />}>
            <Route index element={<DashboardPage />} />
            <Route path="orders" element={<OrdersPage />} />
            <Route path="orders/new" element={<OrderCreatePage />} />
            <Route path="orders/:id" element={<OrderDetailPage />} />
            <Route path="orders/:id/edit" element={<OrderEditPage />} />
            <Route path="clients" element={<ClientsPage />} />
            <Route path="clients/:id" element={<ClientDetailPage />} />
            <Route path="cars" element={<CarsPage />} />
            <Route path="cars/:id" element={<CarDetailPage />} />
            <Route path="inventory" element={<InventoryPage />} />
            <Route path="reminders" element={<RemindersPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  );
}
