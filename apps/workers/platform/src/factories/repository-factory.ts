import { drizzle } from 'drizzle-orm/durable-sqlite';
import { platform_schema } from '../data/schema/index.js';
import { EventRepository } from '../repositories/event.repository.js';
import { WorkspaceProjectionRepository } from '../repositories/workspace-projection.repository.js';
import { JobProjectionRepository } from '../repositories/job-projection.repository.js';

export class RepositoryFactory {
  private _eventRepo?: EventRepository;
  private _workspaceProjectionRepo?: WorkspaceProjectionRepository;
  private _jobProjectionRepo?: JobProjectionRepository;

  constructor(private storage: DurableObjectStorage) {}

  private get db() {
    return drizzle(this.storage, { schema: platform_schema });
  }

  get events(): EventRepository {
    if (!this._eventRepo) {
      this._eventRepo = new EventRepository(this.db);
    }
    return this._eventRepo;
  }

  get workspaceProjection(): WorkspaceProjectionRepository {
    if (!this._workspaceProjectionRepo) {
      this._workspaceProjectionRepo = new WorkspaceProjectionRepository(this.db);
    }
    return this._workspaceProjectionRepo;
  }

  get jobProjection(): JobProjectionRepository {
    if (!this._jobProjectionRepo) {
      this._jobProjectionRepo = new JobProjectionRepository(this.db);
    }
    return this._jobProjectionRepo;
  }
}
