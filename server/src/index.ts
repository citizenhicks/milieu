import { handleApiRequest } from "./api";
import { handleSiteRequest } from "./site";
import { Env } from "./env";

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname.startsWith("/v1/")) {
      return handleApiRequest(request, env);
    }
    return handleSiteRequest(request);
  },
};
