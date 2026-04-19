declare namespace Cloudflare {
  interface GlobalProps {
    mainModule: typeof import('./src/index');
  }
}

interface Env {
  PLATFORM: DurableObjectNamespace<import('../actors/platform/src/index').Platform>;
  WORKSPACE: DurableObjectNamespace<import('../actors/workspace/src/index').Workspace>;
}
