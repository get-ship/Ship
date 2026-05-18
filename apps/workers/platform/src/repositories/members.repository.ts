import { eq, sql } from 'drizzle-orm';
import type { DrizzleSqliteDODatabase } from 'drizzle-orm/durable-sqlite';
import { Result } from '@ship-cloud/types';
import { members } from '../data/schema/members.js';
import type { platform_schema } from '../data/schema/index.js';

export type MemberRole = 'owner' | 'member';

export interface Member {
  user_id: string;
  role: string;
  created_at: string;
}

export class MembersRepository {
  constructor(private db: DrizzleSqliteDODatabase<typeof platform_schema>) {}

  async list(): Promise<Result<Member[]>> {
    try {
      const rows = await this.db.select().from(members);
      return Result.ok(rows);
    } catch (error) {
      return Result.fail(
        new Error(`Failed to list members: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }

  async isMember(userId: string): Promise<Result<boolean>> {
    try {
      const rows = await this.db
        .select({ user_id: members.user_id })
        .from(members)
        .where(eq(members.user_id, userId))
        .limit(1);
      return Result.ok(rows.length > 0);
    } catch (error) {
      return Result.fail(
        new Error(`Failed to check member: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }

  async count(): Promise<Result<number>> {
    try {
      const rows = await this.db.select({ n: sql<number>`COUNT(*)` }).from(members);
      return Result.ok(Number(rows[0]?.n ?? 0));
    } catch (error) {
      return Result.fail(
        new Error(`Failed to count members: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }

  async add(userId: string, role: MemberRole): Promise<Result<void>> {
    try {
      await this.db
        .insert(members)
        .values({ user_id: userId, role, created_at: new Date().toISOString() })
        .onConflictDoNothing();
      return Result.ok();
    } catch (error) {
      return Result.fail(
        new Error(`Failed to add member: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }

  async remove(userId: string): Promise<Result<void>> {
    try {
      await this.db.delete(members).where(eq(members.user_id, userId));
      return Result.ok();
    } catch (error) {
      return Result.fail(
        new Error(`Failed to remove member: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }

  /**
   * Atomic-from-the-DO-instance's-perspective claim: if no members exist,
   * insert the caller as 'owner' and report claimed=true. Otherwise check
   * membership and report claimed=false. Returns authorized=true iff the
   * caller ends up as a member.
   *
   * Safe against concurrent calls because Durable Object request handling
   * is single-threaded — only one claimOrCheck runs at a time per DO.
   */
  async claimOrCheck(userId: string): Promise<Result<{ authorized: boolean; claimed: boolean }>> {
    try {
      const countRes = await this.count();
      if (countRes.isFailure) return Result.fail(new Error(countRes.errorMessage));

      if (countRes.value === 0) {
        const add = await this.add(userId, 'owner');
        if (add.isFailure) return Result.fail(new Error(add.errorMessage));
        return Result.ok({ authorized: true, claimed: true });
      }

      const member = await this.isMember(userId);
      if (member.isFailure) return Result.fail(new Error(member.errorMessage));
      return Result.ok({ authorized: member.value, claimed: false });
    } catch (error) {
      return Result.fail(
        new Error(`Failed to claim or check membership: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }
}
