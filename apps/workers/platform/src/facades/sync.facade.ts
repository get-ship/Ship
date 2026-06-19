import { RpcTarget } from 'cloudflare:workers';
import type { EventEnvelope, SyncPushResponse, SyncPullResponse } from '@ship-cloud/types';
import type { RepositoryFactory } from '../factories/repository-factory.js';

export class SyncFacade extends RpcTarget {
  constructor(private repos: RepositoryFactory) {
    super();
  }

  async push(events: EventEnvelope[]): Promise<SyncPushResponse> {
    const result = await this.repos.events.push(events);
    if (result.isFailure) {
      throw new Error(result.errorMessage);
    }

    // Apply projection updates for each accepted event
    for (const event of events) {
      await this.repos.jobProjection.applyEvent(event);
    }

    return { accepted: result.value };
  }

  async pull(since?: string, limit?: number): Promise<SyncPullResponse> {
    const result = await this.repos.events.pull(since, limit);
    if (result.isFailure) {
      throw new Error(result.errorMessage);
    }
    return result.value;
  }
}
