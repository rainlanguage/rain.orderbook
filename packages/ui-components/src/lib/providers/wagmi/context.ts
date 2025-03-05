import { getContext, setContext } from 'svelte'
import { readable } from 'svelte/store'
import type { Config } from '@wagmi/core'
import type { AppKit } from '@reown/appkit'
import type { Readable } from 'svelte/store'

const _wagmiContextKey = '$$wagmiConfig'
const _appKitContextKey = '$$appKitModal'

/** Retrieves the Wagmi Config from Svelte's context */
export const getWagmiContext = (): Config => {
  const config = getContext(_wagmiContextKey)
  if (!config) {
    throw new Error(
      'No Wagmi Config was found in Svelte context. Did you forget to wrap your component with WagmiProvider?',
    )
  }
  return config as Config
}

/** Sets the Wagmi Config on Svelte's context */
export const setWagmiContext = (config: Config): void => {
  setContext(_wagmiContextKey, config)
}

/** Retrieves the AppKit from Svelte's context */
export const getAppKitContext = (): AppKit => {
  const appKit = getContext(_appKitContextKey)
  if (!appKit) {
    throw new Error(
      'No AppKit was found in Svelte context. Did you forget to wrap your component with WagmiProvider?',
    )
  }
  return appKit as AppKit
}

/** Sets the AppKit on Svelte's context */
export const setAppKitContext = (appKit: AppKit): void => {
  setContext(_appKitContextKey, appKit)
}

const _isLoadingContextKey = '$$isLoading'

/** Retrieves the loading state from Svelte's context */
export const getIsLoadingContext = (): Readable<boolean> => {
  try {
    const isLoading = getContext<Readable<boolean> | undefined>(
      _isLoadingContextKey,
    )
    return isLoading ? isLoading : readable(false)
  } catch (error) {
    return readable(false)
  }
}

/** Sets the loading state on Svelte's context */
export const setIsLoadingContext = (isLoading: Readable<boolean>): void => {
  setContext(_isLoadingContextKey, isLoading)
}