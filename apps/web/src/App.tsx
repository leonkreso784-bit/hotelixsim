import type { ApiClient } from "@hotelixsim/api-client";
import { indexLabel, t } from "@hotelixsim/i18n";
import { useQuery } from "@tanstack/react-query";
import { formatIndex } from "./format";

const PROFILE = "C";

export function App({ client }: { client: ApiClient }) {
  const health = useQuery({
    queryKey: ["health"],
    queryFn: async () => {
      const { data, error, response } = await client.GET("/v1/health");
      if (data) return data;
      if (response.status === 503 && error) return error;
      throw new Error(`health ${response.status}`);
    },
  });
  const indices = useQuery({
    queryKey: ["indices", PROFILE],
    queryFn: async () => {
      const { data, response } = await client.GET("/v1/demo/profiles/{profile}/indices", {
        params: { path: { profile: PROFILE } },
      });
      if (!data) throw new Error(`indices ${response.status}`);
      return data;
    },
  });

  return (
    <main>
      <h1>{t("app.title")}</h1>
      <p>
        {health.isPending
          ? t("status.loading")
          : health.isError
            ? t("status.api.error")
            : health.data.database === "ok"
              ? t("status.db.ok")
              : t("status.db.error")}
      </p>
      {health.data?.events != null && <p>{t("status.events", { count: health.data.events })}</p>}
      <h2>{t("profile.heading", { profile: PROFILE })}</h2>
      {indices.isPending && <p>{t("status.indices.loading")}</p>}
      {indices.isError && <p>{t("status.indices.error")}</p>}
      <ul>
        {indices.data?.indices.map((i) => (
          <li key={i.code}>
            <span>{indexLabel(i.code)}</span> <strong>{formatIndex(i.hundredths)}</strong>
          </li>
        ))}
      </ul>
    </main>
  );
}
