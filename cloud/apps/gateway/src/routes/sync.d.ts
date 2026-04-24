import { Hono } from 'hono';
declare const sync: Hono<{
    Bindings: Env;
    Variables: {
        projectId: string;
    };
}, import("hono/types").BlankSchema, "/">;
export { sync as syncRoutes };
