import type { ApiClient } from "@hotelixsim/api-client";
import { createContext, type ReactNode, useContext } from "react";

const ApiContext = createContext<ApiClient | null>(null);

export function ApiProvider({ client, children }: { client: ApiClient; children: ReactNode }) {
  return <ApiContext value={client}>{children}</ApiContext>;
}

export function useApi(): ApiClient {
  const client = useContext(ApiContext);
  if (!client) throw new Error("ApiProvider nedostaje");
  return client;
}
