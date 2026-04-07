// Hook for consuming the ship/event notification stream.
// Events are sourced via ship.onEvent() from the ShipClient (connected in ShipClientProvider).

import { useState, useEffect, useCallback } from 'react'
import { useShipClient } from '#/features/studio/ShipClientProvider'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'

/** Shape of a ship/event notification payload. */
export interface EventEnvelope {
  id: string
  event_type: string
  entity_id: string | null
  actor: string | null
  payload_json: string | null
  workspace_id: string | null
  created_at: string
}

export interface UseEventStreamReturn {
  events: EventEnvelope[]
  isConnected: boolean
  clearEvents: () => void
}

const MAX_EVENTS = 200

/**
 * Returns the live ship/event stream from the daemon SSE channel.
 * Ring buffer: max 200 events, most-recent-first.
 */
export function useEventStream(): UseEventStreamReturn {
  const client = useShipClient()
  const { connected } = useDaemon()
  const [events, setEvents] = useState<EventEnvelope[]>([])

  useEffect(() => {
    return client.onEvent((e) => {
      const envelope: EventEnvelope = {
        id: e.id,
        event_type: e.event_type,
        entity_id: e.entity_id ?? null,
        actor: e.actor_id ?? null,
        payload_json: e.payload_json ?? null,
        workspace_id: e.workspace_id ?? null,
        created_at: e.created_at,
      }
      setEvents((prev) => {
        const next = [envelope, ...prev]
        return next.length > MAX_EVENTS ? next.slice(0, MAX_EVENTS) : next
      })
    })
  }, [client])

  const clearEvents = useCallback(() => setEvents([]), [])

  return { events, isConnected: connected, clearEvents }
}
