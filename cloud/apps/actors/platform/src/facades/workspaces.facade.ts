import { RpcTarget } from 'cloudflare:workers';
import type { RepositoryFactory } from '../factories/repository-factory.js';

export class WorkspacesFacade extends RpcTarget {
  constructor(private repos: RepositoryFactory) {
    super();
  }

  async list() {
    const result = await this.repos.workspaceProjection.list();
    if (result.isFailure) {
      throw new Error(result.errorMessage);
    }
    return result.value;
  }
}
