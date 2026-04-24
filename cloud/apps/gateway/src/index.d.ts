declare const app: import("hono/hono-base").HonoBase<{
    Bindings: Env;
    Variables: {
        projectId: string;
    };
}, ({
    [x: string]: {};
} | import("hono/types").MergeSchemaPath<import("hono/types").BlankSchema, "/api/sync"> | import("hono/types").MergeSchemaPath<import("hono/types").BlankSchema, "/api/workspaces"> | import("hono/types").MergeSchemaPath<import("hono/types").BlankSchema, "/api/jobs">) & {
    "/health": {
        $get: {
            input: {};
            output: {
                status: string;
                timestamp: string;
            };
            outputFormat: "json";
            status: import("hono/utils/http-status").ContentfulStatusCode;
        };
    };
}, "/">;
export default app;
export type AppType = typeof app;
