import { getWagmiContext, getAppKitContext } from './context'
import {
  connected,
  wagmiLoaded,
  chainId,
  signerAddress,
  loading,
  WC,
  disconnectWagmi,
  defaultWagmiConfig,
  initWagmi,
  wagmiConfig,
  appKitModal
} from './wagmiStores'

/**
 * Hook to access Wagmi client from context
 */
export function useWagmiClient() {
  // Get config from context
  const config = getWagmiContext()
  const appKit = getAppKitContext()
  
  // Return a client object with added defaultWagmiConfig
  return {
    // Store references
    signerAddress,
    connected,
    wagmiLoaded,
    chainId,
    loading,
    wagmiConfig,
    appKitModal,
    
    // Methods using the context config
    connect: WC,
    disconnect: disconnectWagmi,
    
    // Configuration functions
    defaultWagmiConfig,
    initWagmi,
    
    // Config access
    config,
    appKit
  }
}