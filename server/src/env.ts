export interface Env {
  DB: D1Database;
  SESSION_TTL_HOURS: string;
  LOGIN_RATE_LIMIT?: string;
  LOGIN_RATE_WINDOW_SECONDS?: string;
}
