import { createApiClient } from "@hotelixsim/api-client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";

const root = document.getElementById("root");
if (!root) throw new Error("nema #root elementa");

createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={new QueryClient()}>
      <App client={createApiClient(window.location.origin)} />
    </QueryClientProvider>
  </StrictMode>,
);
