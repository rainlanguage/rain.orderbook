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
})