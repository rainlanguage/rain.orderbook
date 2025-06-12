import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import DepositModal from '$lib/components/DepositModal.svelte';
import { readContract, switchChain } from '@wagmi/core';
import type { ComponentProps } from 'svelte';
import type { Hex } from 'viem';
import truncateEthAddress from 'truncate-eth-address';
import { mockWeb3Config } from '$lib/__mocks__/mockWeb3Config';

type ModalProps = ComponentProps<DepositModal>;

const { mockAppKitModalStore, mockConnectedStore, mockWagmiConfigStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

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

describe('DepositModal', () => {
	const mockVault = {
		token: {
			address: '0x123',
			symbol: 'TEST',
			decimals: '18'
		},
		vaultId: '1',
		balance: BigInt(1)
	};

	const mockOnSubmit = vi.fn();

	const defaultProps = {
		open: true,
		onSubmit: mockOnSubmit,
		args: {
			vault: mockVault,
			chainId: 1,
			rpcUrl: 'https://example.com',
			account: '0x0000000000000000000000000000000000000000',
			config: mockWeb3Config
		}
	} as unknown as ModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		mockOnSubmit.mockClear();

		(readContract as Mock).mockReset();
		(switchChain as Mock).mockReset();
		(readContract as Mock).mockResolvedValue(BigInt(1000000000000000000)); // 1 token by default
	});

	it('renders deposit modal correctly', () => {
		render(DepositModal, defaultProps);
		expect(screen.getByTestId('modal-title')).toHaveTextContent('Deposit');
		expect(screen.getByTestId('deposit-button')).toBeInTheDocument();
	});

	it('handles deposit transaction correctly', async () => {
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByTestId('deposit-button');
		await fireEvent.click(depositButton);

		expect(mockOnSubmit).toHaveBeenCalledWith(BigInt(1000000000000000000));
	});

	it('shows error when amount exceeds balance', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(500000000000000000)); // 0.5 tokens
		render(DepositModal, defaultProps);

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
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(
				screen.getByText(/Switch to .* to check your balance|Failed to switch chain/)
			).toBeInTheDocument();
		});
	});

	it('disables continue button when amount is 0', async () => {
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '0' } });

		const continueButton = screen.getByTestId('deposit-button');
		expect(continueButton).toBeDisabled();
	});

	it('disables continue button when amount exceeds balance', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(500000000000000000)); // 0.5 tokens
		render(DepositModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const continueButton = screen.getByTestId('deposit-button');
		expect(continueButton).toBeDisabled();
	});

	it('handles zero user balance correctly', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(0));

		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
			expect(screen.getByText(/0 TEST|0.0 TEST/)).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '0.1' } });

		const depositButton = screen.getByTestId('deposit-button');
		expect(depositButton).toBeDisabled();
		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('displays user balance correctly', async () => {
		const userBalanceAmount = BigInt(2500000000000000000); // 2.5 tokens
		(readContract as Mock).mockResolvedValue(userBalanceAmount);

		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
			expect(screen.getByText('2.5 TEST')).toBeInTheDocument();
		});
	});

	it('shows error message when getUserBalance fails', async () => {
		vi.mocked(readContract).mockRejectedValue(new Error('Failed to get balance'));

		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Failed to get user balance.')).toBeInTheDocument();
		});
	});

	it('shows wallet address in truncated form', async () => {
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(
				screen.getByText(truncateEthAddress('0x0000000000000000000000000000000000000000'))
			).toBeInTheDocument();
		});
	});

	it('shows the cancel button that closes the modal', async () => {
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Cancel')).toBeInTheDocument();
		});

		const cancelButton = screen.getByText('Cancel');
		await fireEvent.click(cancelButton);

		await waitFor(() => {
			expect(screen.queryByTestId('modal-title')).not.toBeInTheDocument();
		});
	});

	it('shows the WalletConnect button when account is not provided', async () => {
		render(DepositModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				account: null as unknown as Hex
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Connect your wallet to continue.')).toBeInTheDocument();
			expect(screen.queryByTestId('deposit-button')).not.toBeInTheDocument();
		});
	});
});
