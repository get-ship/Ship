import { Hono } from 'hono';
declare const workspaces: Hono<{
    Bindings: Env;
    Variables: {
        projectId: string;
    };
}, import("hono/types").BlankSchema, "/">;
export { workspaces as workspaceRoutes };
