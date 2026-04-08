// React bindings for @ship/sdk's ShipClient.
// Provides useShipQuery<T> for reactive queries backed by ShipClient's internal cache.

import { useState, useEffect } from 'react'
import { useShipClient } from '../ShipClientProvider'

export interface ShipQueryResult<T> {
  data: T | undefined
  error: Error | null
  isLoading: boolean
  isSuccess: boolean
  refetch: () => void
}

const NOOP = () => {}
const EMPTY = Object.freeze({
  data: undefined,
  error: null,
  isLoading: false,
  isSuccess: false,
  refetch: NOOP,
})

/**
 * Subscribe to a ShipClient query and re-render on state changes.
 * Pass null to disable the query (returns empty state immediately).
 */
export function useShipQuery<T>(path: string | null): ShipQueryResult<T> {
  const client = useShipClient()
  const [result, setResult] = useState<ShipQueryResult<T>>(EMPTY as ShipQueryResult<T>)

  useEffect(() => {
    if (!path) {
      setResult(EMPTY as ShipQueryResult<T>)
      return
    }
    const q = client.query<T>(path)
    setResult({
      data: q.data,
      error: q.error,
      isLoading: q.isLoading,
      isSuccess: q.isSuccess,
      refetch: q.refetch,
    })
    return q.subscribe((s) => {
      setResult({
        data: s.data,
        error: s.error,
        isLoading: s.isLoading,
        isSuccess: s.isSuccess,
        refetch: s.refetch,
      })
    })
  }, [client, path])

  return result
}
