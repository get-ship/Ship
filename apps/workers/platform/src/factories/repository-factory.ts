import { drizzle } from 'drizzle-orm/durable-sqlite';
import { platform_schema } from '../data/schema/index.js';
import { EventRepository } from '../repositories/event.repository.js';
import { ProjectProjectionRepository } from '../repositories/project-projection.repository.js';
import { JobProjectionRepository } from '../repositories/job-projection.repository.js';

export class RepositoryFactory {
  private _eventRepo?: EventRepository;
  private _projectProjectionRepo?: ProjectProjectionRepository;
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

  get projectProjection(): ProjectProjectionRepository {
    if (!this._projectProjectionRepo) {
      this._projectProjectionRepo = new ProjectProjectionRepository(this.db);
    }
    return this._projectProjectionRepo;
  }

  get jobProjection(): JobProjectionRepository {
    if (!this._jobProjectionRepo) {
      this._jobProjectionRepo = new JobProjectionRepository(this.db);
    }
    return this._jobProjectionRepo;
  }
}
