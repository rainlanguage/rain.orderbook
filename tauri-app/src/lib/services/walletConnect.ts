import {
  createWeb3Modal,
  defaultWagmiConfig
} from '@web3modal/wagmi'
import { mainnet, polygon } from 'viem/chains'
import { reconnect } from '@wagmi/core'
// import { walletConnect } from '@wagmi/connectors'

// 1. Define constants
const projectId = "634cfe0b2781e2ac78219ca4cb23c13f"

// 2. Create wagmiConfig
const metadata = {
  name: 'rain-ob',
  description: "some desc",
  url: '', // origin must match your domain & subdomain
  icons: ['https://avatars.githubusercontent.com/u/37784886']
}

// // const chains = [mainnet, polygon];
export const config = defaultWagmiConfig({
  chains: [mainnet, polygon], // required
  projectId, // required
  metadata, // required
  enableWalletConnect: true, // Optional - true by default
  enableInjected: false, // Optional - true by default
  enableEIP6963: false, // Optional - true by default
  enableCoinbase: false, // Optional - true by default
  enableSmartAccounts: false,
  enableEmail: false
})
// export const config = createConfig({
//   chains: [mainnet, polygon],
//   transports: {
//     [mainnet.id]: http(),
//     [polygon.id]: http(),
//   },
//   connectors: [
//     walletConnect({ projectId, metadata, showQrModal: false }),
//   ],

// })
reconnect(config)

// 3. Create modal
export const modal = createWeb3Modal({
  wagmiConfig: config,
  projectId,
  enableAnalytics: true, // Optional - defaults to your Cloud configuration
  enableOnramp: true, // Optional - false as default
  allWallets: "HIDE",
  includeWalletIds: [
    "e7c4d26541a7fd84dbdfa9922d3ad21e936e13a7a0e44385d44f006139e44d3b" // walletconnect
  ],
  // excludeWalletIds: [
  //   "c57ca95b47569778a828d19178114f4db188b89b763c899ba0be274e97267d96", // metamask
  //   "4622a2b2d6af1c9844944291e5e7351a6aa24cd7b23099efac1b2fd875da31a0"  // trust wallet
  // ]
})