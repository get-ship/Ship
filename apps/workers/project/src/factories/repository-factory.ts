import { drizzle } from 'drizzle-orm/durable-sqlite';
import { project_schema } from '../data/schema/index.js';
import { EventRepository } from '../repositories/event.repository.js';
import { ActorProjectionRepository } from '../repositories/actor-projection.repository.js';
import { SessionProjectionRepository } from '../repositories/session-projection.repository.js';

export class RepositoryFactory {
  private _eventRepo?: EventRepository;
  private _actorProjectionRepo?: ActorProjectionRepository;
  private _sessionProjectionRepo?: SessionProjectionRepository;

  constructor(private storage: DurableObjectStorage) {}

  private get db() {
    return drizzle(this.storage, { schema: project_schema });
  }

  get events(): EventRepository {
    if (!this._eventRepo) {
      this._eventRepo = new EventRepository(this.db);
    }
    return this._eventRepo;
  }

  get actorProjection(): ActorProjectionRepository {
    if (!this._actorProjectionRepo) {
      this._actorProjectionRepo = new ActorProjectionRepository(this.db);
    }
    return this._actorProjectionRepo;
  }

  get sessionProjection(): SessionProjectionRepository {
    if (!this._sessionProjectionRepo) {
      this._sessionProjectionRepo = new SessionProjectionRepository(this.db);
    }
    return this._sessionProjectionRepo;
  }
}
