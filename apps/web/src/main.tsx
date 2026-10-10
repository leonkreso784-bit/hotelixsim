import { createApiClient } from "@hotelixsim/api-client";
import { QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { ApiProvider } from "./api/ApiContext";
import { createQueryClient } from "./api/queryClient";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("nema #root elementa");

createRoot(root).render(
  <StrictMode>
    <ApiProvider client={createApiClient(window.location.origin)}>
      <QueryClientProvider client={createQueryClient()}>
        <App />
      </QueryClientProvider>
    </ApiProvider>
  </StrictMode>,
);
