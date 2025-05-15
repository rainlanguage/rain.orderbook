import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import TransactionConfirmationModal from '$lib/components/TransactionConfirmationModal.svelte';
import { sendTransaction, switchChain } from '@wagmi/core';
import type { TransactionConfirmationProps } from '@rainlanguage/ui-components';
import type { Chain } from 'viem';
import { mockWeb3Config } from '$lib/__mocks__/mockWeb3Config';

const { mockWagmiConfigStore } = await vi.hoisted(() => import('../lib/__mocks__/stores'));

// const mockChain: Chain = {
// 	id: 1,
// 	name: 'Ethereum',
// 	nativeCurrency: {
// 		name: 'Ether',
// 		symbol: 'ETH',
// 		decimals: 18
// 	},
// 	rpcUrls: {
// 		default: { http: ['https://eth.llamarpc.com'] },
// 		public: { http: ['https://eth.llamarpc.com'] }
// 	},
// 	blockExplorers: {
// 		default: { name: 'Etherscan', url: 'https://etherscan.io' }
// 	}
// };

vi.mock('@wagmi/core', () => ({
	sendTransaction: vi.fn(),
	switchChain: vi.fn()
}));

vi.mock('$lib/stores/wagmi', () => ({
	wagmiConfig: mockWagmiConfigStore
}));

describe('TransactionConfirmationModal', () => {
	const mockCalldata = '0x1234567890abcdef';
	const mockTxHash = '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';

	const defaultProps = {
		open: true,
		args: {
			chainId: 1,
			orderbookAddress: '0x789',
			getCalldataFn: vi.fn().mockResolvedValue(mockCalldata),
			onConfirm: vi.fn()
		}
	} as unknown as TransactionConfirmationProps;

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		vi.mocked(switchChain).mockResolvedValue({} as Chain);
	});

	it('shows awaiting confirmation state initially', () => {
		render(TransactionConfirmationModal, defaultProps);

		expect(screen.getByText('Waiting for wallet confirmation')).toBeInTheDocument();
		expect(screen.getByText('Please confirm this transaction in your wallet.')).toBeInTheDocument();
		expect(screen.getByTestId('transaction-modal')).toBeInTheDocument();
	});

	it('handles successful transaction flow', async () => {
		vi.mocked(switchChain).mockResolvedValue({} as Chain);
		vi.mocked(sendTransaction).mockResolvedValue(mockTxHash as any);

		render(TransactionConfirmationModal, defaultProps);

		expect(switchChain).toHaveBeenCalledWith(mockWeb3Config, { chainId: 1 });
		expect(sendTransaction).toHaveBeenCalledWith(mockWeb3Config, {
			to: '0x789',
			data: mockCalldata
		});
		expect(defaultProps.args.onConfirm).toHaveBeenCalledWith(mockTxHash);
		expect(screen.getByText('Transaction Submitted')).toBeInTheDocument();
		expect(screen.getByText('Transaction has been submitted to the network.')).toBeInTheDocument();
	});

	it('handles chain switch error', async () => {
		const errorMessage = 'Failed to switch chain';
		vi.mocked(switchChain).mockRejectedValue(new Error(errorMessage));

		render(TransactionConfirmationModal, defaultProps);

		expect(screen.getByText('Confirmation failed')).toBeInTheDocument();
		expect(screen.getByText(errorMessage)).toBeInTheDocument();
		expect(screen.getByText('Dismiss')).toBeInTheDocument();
	});

	it('handles transaction rejection', async () => {
		vi.mocked(switchChain).mockResolvedValue({} as Chain);
		vi.mocked(sendTransaction).mockRejectedValue(new Error('User rejected transaction'));

		render(TransactionConfirmationModal, defaultProps);

		expect(screen.getByText('Transaction rejected')).toBeInTheDocument();
		expect(screen.getByText('User rejected transaction')).toBeInTheDocument();
		expect(screen.getByText('Dismiss')).toBeInTheDocument();
	});

	it('handles transaction error', async () => {
		vi.mocked(switchChain).mockResolvedValue({} as Chain);
		vi.mocked(sendTransaction).mockRejectedValue(new Error('Transaction failed'));

		render(TransactionConfirmationModal, defaultProps);

		expect(screen.getByText('Transaction rejected')).toBeInTheDocument();
		expect(screen.getByText('User rejected transaction')).toBeInTheDocument();
		expect(screen.getByText('Dismiss')).toBeInTheDocument();
	});

	it('closes modal when dismiss button is clicked', async () => {
		vi.mocked(switchChain).mockRejectedValue(new Error('Failed to switch chain'));

		const { component } = render(TransactionConfirmationModal, defaultProps);

		const dismissButton = screen.getByText('Dismiss');
		await fireEvent.click(dismissButton);

		expect(component.$$.props.open).toBe(false);
	});

	it('shows correct UI elements for different states', async () => {
		const { rerender } = render(TransactionConfirmationModal, defaultProps);

		// Initial state - awaiting confirmation
		expect(screen.getByTestId('transaction-modal')).toHaveClass(
			'bg-opacity-90',
			'backdrop-blur-sm'
		);
		expect(screen.getByRole('dialog')).toHaveAttribute('data-testid', 'transaction-modal');

		// Success state
		vi.mocked(switchChain).mockResolvedValue({} as Chain);
		vi.mocked(sendTransaction).mockResolvedValue(mockTxHash as any);
		rerender(defaultProps);

		expect(screen.getByText('✅')).toBeInTheDocument();
		expect(screen.queryByText('Dismiss')).not.toBeInTheDocument();

		// Error state
		vi.mocked(switchChain).mockRejectedValue(new Error('Failed to switch chain'));
		rerender(defaultProps);

		expect(screen.getByText('❌')).toBeInTheDocument();
		expect(screen.getByText('Dismiss')).toBeInTheDocument();
	});
});
