export interface PollingReconnectOptions {
  isActive: () => boolean
  disconnect: () => Promise<void>
  reconnect: () => Promise<void>
}

export const reconnectAfterPollingFailure = async ({
  isActive,
  disconnect,
  reconnect,
}: PollingReconnectOptions): Promise<boolean> => {
  if (!isActive()) return false

  await disconnect()
  if (!isActive()) return false

  await reconnect()
  return true
}
