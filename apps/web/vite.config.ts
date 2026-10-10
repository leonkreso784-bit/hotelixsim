import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 5173,
    strictPort: true,
    proxy: { "/v1": "http://127.0.0.1:8080" },
  },
  // Testovi ekrana renderiraju cijeli tijek kroz FakeApi; samostalno traju ~1 s, ali pod punim paralelnim
  // opterećenjem (ili na CI-ju) zadanih 5 s je jednom isteklo (10.10., „obračunat lipanj“ 5,4 s).
  test: { environment: "jsdom", testTimeout: 15_000 },
});
