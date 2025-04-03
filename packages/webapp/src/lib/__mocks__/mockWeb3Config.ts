import { createConfig, http, fallback, type Config } from '@wagmi/core';
import { mock } from '@wagmi/connectors';
import { mainnet } from '@wagmi/core/chains';

export const mockWeb3Config: Config = createConfig({
	multiInjectedProviderDiscovery: true,
	chains: [mainnet],
	connectors: [
		mock({
			accounts: ['0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11']
		})
	],
	transports: {
		[mainnet.id]: fallback([http(), http('https://mainnet.infura.io/v3/')])
	}
});
