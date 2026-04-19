import { RpcTarget } from 'cloudflare:workers';
import type { RepositoryFactory } from '../factories/repository-factory.js';

export class JobsFacade extends RpcTarget {
  constructor(private repos: RepositoryFactory) {
    super();
  }

  async list() {
    const result = await this.repos.jobProjection.list();
    if (result.isFailure) {
      throw new Error(result.errorMessage);
    }
    return result.value;
  }
}
