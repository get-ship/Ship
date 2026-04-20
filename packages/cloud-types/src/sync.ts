import type { EventEnvelope } from './events.js';

/** Push events request */
export interface SyncPushRequest {
  events: EventEnvelope[];
}

/** Push events response */
export interface SyncPushResponse {
  accepted: number;
}

/** Pull events request (query params) */
export interface SyncPullRequest {
  since?: string; // cursor: event ID or ISO timestamp
  limit?: number;
}

/** Pull events response */
export interface SyncPullResponse {
  events: EventEnvelope[];
  cursor: string | null; // null means no more events
}
