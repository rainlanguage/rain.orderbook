import { describe, it, expect, vi, beforeEach } from 'vitest';
import { handleWalletConfirmation } from '../lib/services/handleWalletConfirmation';
import { sendTransaction, switchChain } from '@wagmi/core';
import { mockWeb3Config } from '$lib/__mocks__/mockWeb3Config';
import type { SgOrder } from '@rainlanguage/orderbook';

const { mockWagmiConfigStore } = await vi.hoisted(() => import('../lib/__mocks__/stores'));

vi.mock('@wagmi/core', () => ({
	sendTransaction: vi.fn(),
	switchChain: vi.fn()
}));

vi.mock('$lib/stores/wagmi', () => ({
	wagmiConfig: mockWagmiConfigStore
}));

describe('handleWalletConfirmation', () => {
	const mockCalldata = '0x1234567890abcdef';
	const mockTxHash = '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';

	const mockOrder: SgOrder = {
		id: '0x1',
		orderBytes: '0x2',
		orderHash: '0x3',
		owner: '0x4',
		outputs: [],
		inputs: [],
		orderbook: { id: '0x5' },
		active: true,
		timestampAdded: '1234567890',
		addEvents: [],
		trades: [],
		removeEvents: []
	};

	const defaultArgs = {
		chainId: 1,
		orderbookAddress: '0x789' as `0x${string}`,
		calldata: mockCalldata,
		onConfirm: vi.fn(),
		order: mockOrder
	};

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		vi.mocked(switchChain).mockResolvedValue({} as any);
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		vi.mocked(sendTransaction).mockResolvedValue(mockTxHash as any);
	});

	it('handles successful transaction flow', async () => {
		const result = await handleWalletConfirmation(defaultArgs);

		expect(switchChain).toHaveBeenCalledWith(mockWeb3Config, { chainId: 1 });
		expect(sendTransaction).toHaveBeenCalledWith(mockWeb3Config, {
			to: '0x789',
			data: mockCalldata
		});
		expect(defaultArgs.onConfirm).toHaveBeenCalledWith(mockTxHash);
		expect(result).toEqual({
			state: { status: 'confirmed' },
			hash: mockTxHash
		});
	});

	it('handles chain switch error', async () => {
		const errorMessage = 'Failed to switch chain';
		vi.mocked(switchChain).mockRejectedValue(new Error(errorMessage));

		const result = await handleWalletConfirmation(defaultArgs);

		expect(switchChain).toHaveBeenCalledWith(mockWeb3Config, { chainId: 1 });
		expect(sendTransaction).not.toHaveBeenCalled();
		expect(defaultArgs.onConfirm).not.toHaveBeenCalled();
		expect(result).toEqual({
			state: {
				status: 'error',
				reason: errorMessage
			}
		});
	});

	it('handles transaction rejection', async () => {
		vi.mocked(sendTransaction).mockRejectedValue(new Error('User rejected transaction'));

		const result = await handleWalletConfirmation(defaultArgs);

		expect(switchChain).toHaveBeenCalledWith(mockWeb3Config, { chainId: 1 });
		expect(sendTransaction).toHaveBeenCalledWith(mockWeb3Config, {
			to: '0x789',
			data: mockCalldata
		});
		expect(defaultArgs.onConfirm).not.toHaveBeenCalled();
		expect(result).toEqual({
			state: {
				status: 'rejected',
				reason: 'User rejected transaction'
			}
		});
	});

	it('handles non-Error chain switch failure', async () => {
		vi.mocked(switchChain).mockRejectedValue('Unknown error');

		const result = await handleWalletConfirmation(defaultArgs);

		expect(switchChain).toHaveBeenCalledWith(mockWeb3Config, { chainId: 1 });
		expect(sendTransaction).not.toHaveBeenCalled();
		expect(defaultArgs.onConfirm).not.toHaveBeenCalled();
		expect(result).toEqual({
			state: {
				status: 'error',
				reason: 'Failed to switch chain'
			}
		});
	});

	it('handles transaction failure', async () => {
		vi.mocked(sendTransaction).mockRejectedValue(new Error('Transaction failed'));

		const result = await handleWalletConfirmation(defaultArgs);

		expect(switchChain).toHaveBeenCalledWith(mockWeb3Config, { chainId: 1 });
		expect(sendTransaction).toHaveBeenCalledWith(mockWeb3Config, {
			to: '0x789',
			data: mockCalldata
		});
		expect(defaultArgs.onConfirm).not.toHaveBeenCalled();
		expect(result).toEqual({
			state: {
				status: 'rejected',
				reason: 'User rejected transaction'
			}
		});
	});
});

describe('handleWalletConfirmation with varied inputs and scenarios', () => {
	const mockCalldata = '0x1234567890abcdef';
	const mockTxHash = '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';

	const mockOrder: SgOrder = {
		id: '0x1',
		orderBytes: '0x2',
		orderHash: '0x3',
		owner: '0x4',
		outputs: [],
		inputs: [],
		orderbook: { id: '0x5' },
		active: true,
		timestampAdded: '1234567890',
		addEvents: [],
		trades: [],
		removeEvents: []
	};

	const getArgs = (chainId: number) => ({
		chainId,
		orderbookAddress: '0x789' as `0x${string}`,
		calldata: mockCalldata,
		onConfirm: vi.fn(),
		order: mockOrder
	});

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		vi.mocked(switchChain).mockResolvedValue({} as any);
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		vi.mocked(sendTransaction).mockResolvedValue(mockTxHash as any);
	});

	it('handles successful transaction with a different chain ID', async () => {
		const specificChainId = 5;
		const args = getArgs(specificChainId);
		const result = await handleWalletConfirmation(args);

		expect(switchChain).toHaveBeenCalledWith(mockWeb3Config, { chainId: specificChainId });
		expect(sendTransaction).toHaveBeenCalledWith(mockWeb3Config, {
			to: '0x789',
			data: mockCalldata
		});
		expect(args.onConfirm).toHaveBeenCalledWith(mockTxHash);
		expect(result).toEqual({
			state: { status: 'confirmed' },
			hash: mockTxHash
		});
	});

	it('handles network failure during transaction sending', async () => {
		const args = getArgs(1);
		const networkErrorMessage = 'Network connection lost';
		vi.mocked(sendTransaction).mockRejectedValue(new Error(networkErrorMessage));

		const result = await handleWalletConfirmation(args);

		expect(switchChain).toHaveBeenCalledWith(mockWeb3Config, { chainId: 1 });
		expect(sendTransaction).toHaveBeenCalledWith(mockWeb3Config, {
			to: '0x789',
			data: mockCalldata
		});
		expect(args.onConfirm).not.toHaveBeenCalled();
		// The current implementation catches all errors from sendTransaction as 'rejected'
		// If specific network error handling is desired, the main function needs adjustment.
		expect(result).toEqual({
			state: {
				status: 'rejected',
				reason: 'User rejected transaction' // This is because any error in sendTransaction leads to this
			}
		});
	});

	it('handles specific error type (e.g., insufficient funds) from wallet provider', async () => {
		const args = getArgs(1);
		// Simulate a specific error. Note: The actual error objects might differ by provider.
		// For this test, we'll use a generic error with a message.
		const specificError = new Error('Insufficient funds for transaction');
		vi.mocked(sendTransaction).mockRejectedValue(specificError);

		const result = await handleWalletConfirmation(args);

		expect(switchChain).toHaveBeenCalledWith(mockWeb3Config, { chainId: 1 });
		expect(sendTransaction).toHaveBeenCalledWith(mockWeb3Config, {
			to: '0x789',
			data: mockCalldata
		});
		expect(args.onConfirm).not.toHaveBeenCalled();
		// As with network errors, the current implementation has a general catch-all.
		expect(result).toEqual({
			state: {
				status: 'rejected',
				reason: 'User rejected transaction'
			}
		});
	});
});
