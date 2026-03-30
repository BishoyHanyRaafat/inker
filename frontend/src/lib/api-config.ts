import type { CreateClientConfig } from "./api/client.gen";

// Token management
const TOKEN_KEY = "inker_access_token";
const REFRESH_TOKEN_KEY = "inker_refresh_token";

export function getAccessToken(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(TOKEN_KEY);
}

export function getRefreshToken(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(REFRESH_TOKEN_KEY);
}

export function setTokens(accessToken: string, refreshToken: string): void {
  localStorage.setItem(TOKEN_KEY, accessToken);
  localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken);
}

export function clearTokens(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(REFRESH_TOKEN_KEY);
}

export function isAuthenticated(): boolean {
  const token = getAccessToken();
  if (!token) return false;

  try {
    const payload = JSON.parse(atob(token.split(".")[1]));
    return payload.exp * 1000 > Date.now();
  } catch {
    return false;
  }
}

/**
 * This function is called by the generated client to get the initial configuration.
 * The @hey-api/client-fetch plugin expects this export.
 */
export const createClientConfig: CreateClientConfig = (config) => ({
  ...config,
  baseUrl: process.env.NEXT_PUBLIC_API_URL || "http://localhost:3000",
  headers: {
    ...config?.headers,
  },
  // Custom fetch wrapper to add auth token
  fetch: (input, init) => {
    const token = getAccessToken();
    // Important for cookie-based auth across different frontend/backend origins:
    // make sure the browser will accept Set-Cookie and send cookies on requests.
    const credentials: RequestCredentials = "include";
    
    // Check if input is a Request object (headers would be embedded there)
    if (input instanceof Request) {
      // Clone the request and add auth header if we have a token
      if (token) {
        const newHeaders = new Headers(input.headers);
        newHeaders.set("Authorization", `Bearer ${token}`);
        return fetch(new Request(input, { headers: newHeaders, credentials }));
      }
      // Ensure cookies are included even without auth header
      return fetch(new Request(input, { credentials }));
    }
    
    // input is a string URL, headers are in init
    const existingHeaders = init?.headers;
    let headers: HeadersInit;
    
    if (existingHeaders instanceof Headers) {
      headers = new Headers(existingHeaders);
      if (token) {
        headers.set("Authorization", `Bearer ${token}`);
      }
    } else if (Array.isArray(existingHeaders)) {
      headers = new Headers(existingHeaders);
      if (token) {
        headers.set("Authorization", `Bearer ${token}`);
      }
    } else if (existingHeaders && typeof existingHeaders === "object") {
      headers = {
        ...existingHeaders,
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      };
    } else {
      headers = token ? { Authorization: `Bearer ${token}` } : {};
    }
    
    return fetch(input, {
      ...init,
      headers,
      credentials,
    });
  },
});
