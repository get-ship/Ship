import { DurableObject } from 'cloudflare:workers';
import { initializeProjectDB } from '../data/init.js';
import { RepositoryFactory } from '../factories/index.js';
import { SyncFacade, ActorsFacade, SessionsFacade } from '../facades/index.js';

export class Project extends DurableObject {
  private _repos?: RepositoryFactory;
  private _sync?: SyncFacade;
  private _actors?: ActorsFacade;
  private _sessions?: SessionsFacade;

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);

    ctx.blockConcurrencyWhile(async () => {
      await initializeProjectDB(ctx.storage);
    });
  }

  // ===== PUBLIC FACADES (RPC Entry Points) =====

  async sync() {
    if (!this._sync) {
      this._sync = new SyncFacade(this.repos);
    }
    return this._sync;
  }

  async actors() {
    if (!this._actors) {
      this._actors = new ActorsFacade(this.repos);
    }
    return this._actors;
  }

  async sessions() {
    if (!this._sessions) {
      this._sessions = new SessionsFacade(this.repos);
    }
    return this._sessions;
  }

  // ===== INTERNAL DEPENDENCIES =====

  private get repos() {
    if (!this._repos) {
      this._repos = new RepositoryFactory(this.ctx.storage);
    }
    return this._repos;
  }
}
