import { defineConfig } from "@hey-api/openapi-ts";

export default defineConfig({
  input: "./openapi.json",
  output: {
    path: "./src/lib/api",
  },
  plugins: [
    "@hey-api/typescript",
    "@hey-api/sdk",
    {
      name: "@hey-api/client-fetch",
      // Path relative to the output directory (./src/lib/api)
      runtimeConfigPath: "../api-config",
    },
  ],
});

