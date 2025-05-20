import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import WithdrawModal from '$lib/components/WithdrawModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import { readContract, switchChain } from '@wagmi/core';
import type { ComponentProps } from 'svelte';
import { getVaultWithdrawCalldata, type SgVault } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import truncateEthAddress from 'truncate-eth-address';
import { mockWeb3Config } from '$lib/__mocks__/mockWeb3Config';

type ModalProps = ComponentProps<WithdrawModal>;

const { mockAppKitModalStore, mockConnectedStore, mockWagmiConfigStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

vi.mock('@rainlanguage/orderbook', () => ({
	getVaultWithdrawCalldata: vi.fn().mockResolvedValue({ value: '0xghi' })
}));

vi.mock('../lib/stores/wagmi', () => ({
	appKitModal: mockAppKitModalStore,
	connected: mockConnectedStore,
	wagmiConfig: mockWagmiConfigStore
}));

vi.mock('@wagmi/core', async (importOriginal) => {
	const original = (await importOriginal()) as object;
	return {
		...original,
		readContract: vi.fn(),
		switchChain: vi.fn().mockResolvedValue({ id: 1 })
	};
});

describe('WithdrawModal', () => {
	const mockVault = {
		token: {
			address: '0x123',
			symbol: 'TEST',
			decimals: '18'
		},
		vaultId: '1',
		balance: BigInt(1000000000000000000) // 1 token
	};

	const defaultProps = {
		open: true,
		args: {
			vault: mockVault,
			chainId: 1,
			rpcUrl: 'https://example.com',
			onDepositOrWithdraw: vi.fn(),
			account: '0x0000000000000000000000000000000000000000'
		}
	} as unknown as ModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(transactionStore, 'reset');

		(readContract as Mock).mockReset();
		(switchChain as Mock).mockReset();
		(readContract as Mock).mockResolvedValue(BigInt(1000000000000000000)); // 1 token by default
	});

	it('renders withdraw modal correctly', () => {
		render(WithdrawModal, defaultProps);
		expect(screen.getByTestId('modal-title')).toHaveTextContent('Withdraw');
		expect(screen.getByTestId('withdraw-button')).toBeInTheDocument();
	});

	it('handles withdraw transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleDepositOrWithdrawTransaction');
		render(WithdrawModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const withdrawButton = screen.getByTestId('withdraw-button');
		await fireEvent.click(withdrawButton);

		expect(handleTransactionSpy).toHaveBeenCalledWith({
			config: mockWeb3Config,
			transactionCalldata: '0xghi',
			action: 'withdraw',
			chainId: 1,
			vault: mockVault,
			subgraphUrl: undefined
		});
	});

	it('shows error when amount exceeds balance', async () => {
		const mockVaultWithBalance = {
			...mockVault,
			balance: BigInt(500000000000000000) // 0.5 tokens
		};

		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				vault: mockVaultWithBalance as unknown as SgVault
			}
		});

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('shows chain switch error when switching fails', async () => {
		(switchChain as Mock).mockRejectedValue(new Error('Failed to switch chain'));
		render(WithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText(/Switch to .* to check your balance/)).toBeInTheDocument();
		});
	});

	it('disables continue button when amount is 0', async () => {
		render(WithdrawModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '0' } });

		const continueButton = screen.getByTestId('withdraw-button');
		expect(continueButton).toBeDisabled();
	});

	it('disables continue button when amount exceeds balance', async () => {
		const mockVaultWithBalance = {
			...mockVault,
			balance: BigInt(500000000000000000) // 0.5 tokens
		};

		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				vault: mockVaultWithBalance as unknown as SgVault
			}
		});

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const continueButton = screen.getByTestId('withdraw-button');
		expect(continueButton).toBeDisabled();
	});

	it('shows loading state while checking calldata', async () => {
		vi.mocked(getVaultWithdrawCalldata).mockImplementationOnce(
			() => new Promise((resolve) => setTimeout(() => resolve({ value: '0xghi', error: undefined }), 100))
		);

		render(WithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const withdrawButton = screen.getByTestId('withdraw-button');
		await fireEvent.click(withdrawButton);

		await waitFor(() => {
			const button = screen.getByTestId('withdraw-button');
			expect(button).toHaveTextContent('Checking...');
		});
	});

	it('handles failed calldata fetch', async () => {
		vi.mocked(getVaultWithdrawCalldata).mockRejectedValueOnce(new Error('Failed to fetch'));

		render(WithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const withdrawButton = screen.getByTestId('withdraw-button');
		await fireEvent.click(withdrawButton);

		await waitFor(() => {
			expect(screen.getByText('Failed to get calldata.')).toBeInTheDocument();
		});
	});

	it('handles zero vault balance correctly', async () => {
		const mockVaultWithZeroBalance = {
			...mockVault,
			balance: BigInt(0)
		};

		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				vault: mockVaultWithZeroBalance as unknown as SgVault
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Balance of vault')).toBeInTheDocument();
			expect(screen.getByText('0 TEST')).toBeInTheDocument();
		});

		const withdrawInput = screen.getByRole('textbox');
		await fireEvent.input(withdrawInput, { target: { value: '0.1' } });

		const withdrawButton = screen.getByTestId('withdraw-button');
		expect(withdrawButton).toBeDisabled();
		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('displays vault balance correctly', async () => {
		const mockVaultWithBalance = {
			...mockVault,
			balance: BigInt(3700000000000000000) // 3.7 tokens
		};

		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				vault: mockVaultWithBalance as unknown as SgVault
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Balance of vault')).toBeInTheDocument();
			expect(screen.getByText('3.7 TEST')).toBeInTheDocument();
		});
	});

	it('shows error message when getUserBalance fails', async () => {
		vi.mocked(readContract).mockRejectedValue(new Error('Failed to get balance'));

		render(WithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Failed to get user balance.')).toBeInTheDocument();
		});
	});

	it('shows wallet address in truncated form', async () => {
		render(WithdrawModal, defaultProps);

		await waitFor(() => {
			expect(
				screen.getByText(truncateEthAddress('0x0000000000000000000000000000000000000000'))
			).toBeInTheDocument();
		});
	});

	it('shows the cancel button that closes the modal', async () => {
		render(WithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Cancel')).toBeInTheDocument();
		});

		const cancelButton = screen.getByText('Cancel');
		await fireEvent.click(cancelButton);
		await waitFor(() => {
			expect(transactionStore.reset).not.toHaveBeenCalled();
		});
	});

	it('shows the WalletConnect button when account is not provided', async () => {
		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				account: null as unknown as Hex
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Connect your wallet to continue.')).toBeInTheDocument();
			expect(screen.queryByTestId('withdraw-button')).not.toBeInTheDocument();
		});
	});
}); 