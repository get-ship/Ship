import { DurableObject } from 'cloudflare:workers';
import { initializePlatformDB } from '../data/init.js';
import { RepositoryFactory } from '../factories/index.js';
import { SyncFacade, ProjectsFacade, JobsFacade } from '../facades/index.js';

export class Platform extends DurableObject {
  private _repos?: RepositoryFactory;
  private _sync?: SyncFacade;
  private _projects?: ProjectsFacade;
  private _jobs?: JobsFacade;

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);

    ctx.blockConcurrencyWhile(async () => {
      await initializePlatformDB(ctx.storage);
    });
  }

  // ===== PUBLIC FACADES (RPC Entry Points) =====

  async sync() {
    if (!this._sync) {
      this._sync = new SyncFacade(this.repos);
    }
    return this._sync;
  }

  async projects() {
    if (!this._projects) {
      this._projects = new ProjectsFacade(this.repos);
    }
    return this._projects;
  }

  async jobs() {
    if (!this._jobs) {
      this._jobs = new JobsFacade(this.repos);
    }
    return this._jobs;
  }

  // ===== INTERNAL DEPENDENCIES =====

  private get repos() {
    if (!this._repos) {
      this._repos = new RepositoryFactory(this.ctx.storage);
    }
    return this._repos;
  }
}
