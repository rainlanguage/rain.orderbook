import { defaultConfig } from '$lib/stores/wagmi';
import { injected, walletConnect } from '@wagmi/connectors';
import { PUBLIC_WALLETCONNECT_PROJECT_ID } from '$env/static/public';
import { supportedChainsList } from '$lib/chains';

export const initWallet = async (): Promise<string | null> => {
	try {
		const erckit = defaultConfig({
			appName: 'Rain Language',
			connectors: [injected(), walletConnect({ projectId: PUBLIC_WALLETCONNECT_PROJECT_ID })],
			chains: [...supportedChainsList],
			projectId: PUBLIC_WALLETCONNECT_PROJECT_ID
		});
		await erckit.init();
		return null;
	} catch (error) {
		return `Failed to initialize wallet connection: ${error instanceof Error ? error.message : 'Unknown error'}. Please try again or check console.`;
	}
};
