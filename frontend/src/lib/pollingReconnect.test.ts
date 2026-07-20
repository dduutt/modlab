import { describe, expect, it } from 'vitest'
import { reconnectAfterPollingFailure } from './pollingReconnect'

const deferred = () => {
  let resolve!: () => void
  const promise = new Promise<void>((resolvePromise) => {
    resolve = resolvePromise
  })
  return { promise, resolve }
}

describe('reconnectAfterPollingFailure', () => {
  it('disconnects and reconnects while the polling session is still active', async () => {
    const calls: string[] = []

    await reconnectAfterPollingFailure({
      isActive: () => true,
      disconnect: async () => { calls.push('disconnect') },
      reconnect: async () => { calls.push('reconnect') },
    })

    expect(calls).toEqual(['disconnect', 'reconnect'])
  })

  it('does not reconnect when the session is stopped during disconnect', async () => {
    let active = true
    const disconnectStarted = deferred()
    const releaseDisconnect = deferred()
    const calls: string[] = []

    const reconnectPromise = reconnectAfterPollingFailure({
      isActive: () => active,
      disconnect: async () => {
        calls.push('disconnect')
        disconnectStarted.resolve()
        await releaseDisconnect.promise
      },
      reconnect: async () => { calls.push('reconnect') },
    })

    await disconnectStarted.promise
    active = false
    releaseDisconnect.resolve()
    await reconnectPromise

    expect(calls).toEqual(['disconnect'])
  })

  it('does not touch the connection when the session is already stopped', async () => {
    const calls: string[] = []

    await reconnectAfterPollingFailure({
      isActive: () => false,
      disconnect: async () => { calls.push('disconnect') },
      reconnect: async () => { calls.push('reconnect') },
    })

    expect(calls).toEqual([])
  })

  it('does not reconnect when disconnect fails', async () => {
    const closeError = new Error('serial close failed')
    const calls: string[] = []

    await expect(reconnectAfterPollingFailure({
      isActive: () => true,
      disconnect: async () => {
        calls.push('disconnect')
        throw closeError
      },
      reconnect: async () => { calls.push('reconnect') },
    })).rejects.toBe(closeError)

    expect(calls).toEqual(['disconnect'])
  })
})
