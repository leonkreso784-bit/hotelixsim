// test/fakeApi.ts — lažni poslužitelj za testove: rute po metodi i putanji, zapis svih poziva.
export function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });
}

export function problem(status: number, code: string): Response {
  return new Response(JSON.stringify({ type: "about:blank", title: code, status, code }), {
    status,
    headers: { "content-type": "application/problem+json" },
  });
}

type Params = Record<string, string>;
export type Handler = (req: Request, params: Params, body: unknown) => Response | Promise<Response>;

type RouteEntry = { method: string; pattern: RegExp; names: string[]; handler: Handler };

export class FakeApi {
  readonly calls: { method: string; path: string; search: string; body: unknown }[] = [];
  private routes: RouteEntry[] = [];

  /** `path` kao "/v1/hotels/:hotel/situations"; tijelo umjesto funkcije = 200 JSON. Kasnija ruta pobjeđuje. */
  on(method: string, path: string, handlerOrBody: Handler | object): this {
    const names: string[] = [];
    const pattern = new RegExp(
      `^${path.replace(/:(\w+)/g, (_m, name: string) => {
        names.push(name);
        return "([^/]+)";
      })}$`,
    );
    const handler: Handler =
      typeof handlerOrBody === "function" ? (handlerOrBody as Handler) : () => json(200, handlerOrBody);
    this.routes.unshift({ method, pattern, names, handler });
    return this;
  }

  posts(path: string) {
    return this.calls.filter((c) => c.method !== "GET" && c.path === path);
  }

  fetch = async (req: Request): Promise<Response> => {
    const url = new URL(req.url);
    const text = req.method === "GET" ? "" : await req.clone().text();
    const body: unknown = text ? JSON.parse(text) : undefined;
    this.calls.push({ method: req.method, path: url.pathname, search: url.search, body });
    for (const r of this.routes) {
      if (r.method !== req.method) continue;
      const m = r.pattern.exec(url.pathname);
      if (!m) continue;
      const params: Params = {};
      r.names.forEach((n, i) => {
        params[n] = decodeURIComponent(m[i + 1] ?? "");
      });
      return r.handler(req, params, body);
    }
    return problem(404, "unknown_route");
  };
}
