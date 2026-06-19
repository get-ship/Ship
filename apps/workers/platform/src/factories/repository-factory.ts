import { drizzle } from 'drizzle-orm/durable-sqlite';
import { platform_schema } from '../data/schema/index.js';
import { EventRepository } from '../repositories/event.repository.js';
import { JobProjectionRepository } from '../repositories/job-projection.repository.js';
import { MembersRepository } from '../repositories/members.repository.js';

export class RepositoryFactory {
  private _eventRepo?: EventRepository;
  private _jobProjectionRepo?: JobProjectionRepository;
  private _membersRepo?: MembersRepository;

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

  get jobProjection(): JobProjectionRepository {
    if (!this._jobProjectionRepo) {
      this._jobProjectionRepo = new JobProjectionRepository(this.db);
    }
    return this._jobProjectionRepo;
  }

  get members(): MembersRepository {
    if (!this._membersRepo) {
      this._membersRepo = new MembersRepository(this.db);
    }
    return this._membersRepo;
  }
}
