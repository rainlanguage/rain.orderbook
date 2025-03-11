import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import {
	connected,
	wagmiLoaded,
	chainId,
	signerAddress,
	configuredConnectors,
	loading,
	defaultWagmiConfig,
	initWagmi,
	disconnectWagmi
} from './wagmi';
import {
	createConfig,
	disconnect,
	getAccount,
	watchAccount,
	type Config,
	type GetAccountReturnType
} from '@wagmi/core';
import { mainnet, type Chain } from '@wagmi/core/chains';

vi.mock('@wagmi/core', async (importOriginal) => ({
	...(await importOriginal()),
	createConfig: vi.fn(),
	disconnect: vi.fn(),
	getAccount: vi.fn(),
	watchAccount: vi.fn(),
	reconnect: vi.fn(),
	http: vi.fn()
}));

vi.mock('@reown/appkit', () => ({
	createAppKit: vi.fn(() => ({
		open: vi.fn(),
		subscribeEvents: vi.fn()
	}))
}));

describe('wagmi store', () => {
	beforeEach(() => {
		// Reset all stores to initial state
		connected.set(false);
		wagmiLoaded.set(false);
		chainId.set(null);
		signerAddress.set(null);
		configuredConnectors.set([]);
		loading.set(true);
	});

	afterEach(() => {
		vi.clearAllMocks();
	});

	describe('defaultWagmiConfig', () => {
		it('should initialize with correct default values', () => {
			const mockConfig = { chains: [mainnet], subscribe: vi.fn() };
			vi.mocked(createConfig).mockReturnValue(mockConfig as unknown as Config);

			const result = defaultWagmiConfig({
				appName: 'Test App',
				projectId: 'test-project-id',
				connectors: [],
				supportedChains: [mainnet]
			});

			expect(result).toHaveProperty('initWagmi');
			expect(get(wagmiLoaded)).toBe(true);
		});
	});

	describe('initWagmi', () => {
		it('should initialize wallet connection successfully', async () => {
			const mockAccount = {
				address: '0x123' as `0x${string}`,
				addresses: ['0x123'] as readonly `0x${string}`[],
				chainId: 1,
				chain: mainnet,
				connector: undefined,
				isConnected: true,
				isConnecting: false,
				isDisconnected: false,
				isReconnecting: false,
				status: 'connected' as const
			};

			vi.mocked(getAccount).mockReturnValue(
				mockAccount as unknown as GetAccountReturnType<Config, Chain>
			);
			vi.mocked(watchAccount).mockImplementation(() => {
				return () => {};
			});

			await initWagmi();

			expect(get(connected)).toBe(true);
			expect(get(signerAddress)).toBe('0x123');
			expect(get(loading)).toBe(false);
		});

		it('should handle initialization failure', async () => {
			vi.mocked(getAccount).mockImplementation(() => {
				throw new Error('Connection failed');
			});

			await initWagmi();

			expect(get(connected)).toBe(false);
			expect(get(loading)).toBe(false);
		});
	});

	describe('disconnectWagmi', () => {
		it('should disconnect wallet and reset stores', async () => {
			connected.set(true);
			chainId.set(1);
			signerAddress.set('0x123');

			await disconnectWagmi();

			expect(vi.mocked(disconnect)).toHaveBeenCalled();
			expect(get(connected)).toBe(false);
			expect(get(chainId)).toBe(null);
			expect(get(signerAddress)).toBe(null);
			expect(get(loading)).toBe(false);
		});
	});
});
