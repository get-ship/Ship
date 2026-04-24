import { Hono } from 'hono';
declare const jobs: Hono<{
    Bindings: Env;
    Variables: {
        projectId: string;
    };
}, import("hono/types").BlankSchema, "/">;
export { jobs as jobRoutes };
