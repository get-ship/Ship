import { RpcTarget } from 'cloudflare:workers';
import type { RepositoryFactory } from '../factories/repository-factory.js';
import type { Member, MemberRole } from '../repositories/members.repository.js';

export class ProjectsFacade extends RpcTarget {
  constructor(private repos: RepositoryFactory) {
    super();
  }

  // ===== Membership =====

  async listMembers(): Promise<Member[]> {
    const result = await this.repos.members.list();
    if (result.isFailure) throw new Error(result.errorMessage);
    return result.value;
  }

  async isMember(userId: string): Promise<boolean> {
    const result = await this.repos.members.isMember(userId);
    if (result.isFailure) throw new Error(result.errorMessage);
    return result.value;
  }

  async addMember(userId: string, role: MemberRole = 'member'): Promise<void> {
    const result = await this.repos.members.add(userId, role);
    if (result.isFailure) throw new Error(result.errorMessage);
  }

  async removeMember(userId: string): Promise<void> {
    const result = await this.repos.members.remove(userId);
    if (result.isFailure) throw new Error(result.errorMessage);
  }

  /**
   * Authorize a user against this project. If the project has no members
   * yet, the caller is added as 'owner' and authorized. Otherwise returns
   * whether the caller is a member. Per-DO single-threaded execution makes
   * this safe against concurrent claim races.
   */
  async claimOrCheck(userId: string): Promise<{ authorized: boolean; claimed: boolean }> {
    const result = await this.repos.members.claimOrCheck(userId);
    if (result.isFailure) throw new Error(result.errorMessage);
    return result.value;
  }
}
