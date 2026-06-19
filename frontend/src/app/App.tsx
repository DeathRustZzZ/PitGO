import { BrowserRouter, Routes, Route } from "react-router-dom";
import { LandingPage } from "../features/landing/LandingPage";
import { CustomerRegistrationPage } from "../features/customer/pages/CustomerRegistrationPage";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<LandingPage />} />
        <Route path="/register" element={<CustomerRegistrationPage />} />
      </Routes>
    </BrowserRouter>
  );
}
