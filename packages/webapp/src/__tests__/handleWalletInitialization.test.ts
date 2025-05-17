import { describe, it, expect, vi, beforeEach } from 'vitest';
import { initWallet } from '../lib/services/handleWalletInitialization';
import { defaultConfig } from '$lib/stores/wagmi';
import { injected, walletConnect } from '@wagmi/connectors';
import { PUBLIC_WALLETCONNECT_PROJECT_ID } from '$env/static/public';
import { supportedChainsList } from '$lib/chains';

// Mock the dependencies
vi.mock('$lib/stores/wagmi', () => ({
	defaultConfig: vi.fn()
}));

vi.mock('@wagmi/connectors', () => ({
	injected: vi.fn(),
	walletConnect: vi.fn()
}));

vi.mock('$env/static/public', () => ({
	PUBLIC_WALLETCONNECT_PROJECT_ID: 'test-project-id'
}));

vi.mock('$lib/chains', () => ({
	supportedChainsList: ['chain1', 'chain2']
}));

describe('handleWalletInitialization', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	function mockDefaultConfig(mockValue: any) {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		(defaultConfig as any).mockReturnValue(mockValue);
	}
	it('should initialize wallet successfully', async () => {
		const mockErckit = {
			init: vi.fn().mockResolvedValue(undefined)
		};

		mockDefaultConfig(mockErckit);

		const result = await initWallet();

		expect(defaultConfig).toHaveBeenCalledWith({
			appName: 'Rain Language',
			connectors: [injected(), walletConnect({ projectId: PUBLIC_WALLETCONNECT_PROJECT_ID })],
			chains: supportedChainsList,
			projectId: PUBLIC_WALLETCONNECT_PROJECT_ID
		});
		expect(mockErckit.init).toHaveBeenCalled();
		expect(result).toBeNull();
	});

	it('should return error message when initialization fails', async () => {
		const mockErckit = {
			init: vi.fn().mockRejectedValue(new Error('Test error'))
		};

		mockDefaultConfig(mockErckit);

		const result = await initWallet();

		expect(defaultConfig).toHaveBeenCalledWith({
			appName: 'Rain Language',
			connectors: [injected(), walletConnect({ projectId: PUBLIC_WALLETCONNECT_PROJECT_ID })],
			chains: supportedChainsList,
			projectId: PUBLIC_WALLETCONNECT_PROJECT_ID
		});
		expect(mockErckit.init).toHaveBeenCalled();
		expect(result).toBe(
			'Failed to initialize wallet connection: Test error. Please try again or check console.'
		);
	});

	it('should handle unknown errors', async () => {
		const mockErckit = {
			init: vi.fn().mockRejectedValue('Unknown error')
		};

		mockDefaultConfig(mockErckit);

		const result = await initWallet();

		expect(defaultConfig).toHaveBeenCalledWith({
			appName: 'Rain Language',
			connectors: [injected(), walletConnect({ projectId: PUBLIC_WALLETCONNECT_PROJECT_ID })],
			chains: supportedChainsList,
			projectId: PUBLIC_WALLETCONNECT_PROJECT_ID
		});
		expect(mockErckit.init).toHaveBeenCalled();
		expect(result).toBe(
			'Failed to initialize wallet connection: Unknown error. Please try again or check console.'
		);
	});
});
