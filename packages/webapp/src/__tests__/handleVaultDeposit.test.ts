import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	handleVaultDeposit,
	type VaultDepositHandlerDependencies
} from '../lib/services/handleVaultDeposit';
import type { SgVault } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import { waitFor } from '@testing-library/svelte';
import type { TransactionManager } from '@rainlanguage/ui-components';

// Mocks
const mockHandleDepositModal = vi.fn();
const mockHandleTransactionConfirmationModal = vi.fn();
const mockErrToast = vi.fn();
const mockCreateDepositTransaction = vi.fn();
const mockCreateApprovalTransaction = vi.fn();

const mockManager = {
	createDepositTransaction: mockCreateDepositTransaction,
	createApprovalTransaction: mockCreateApprovalTransaction
};

const mockVault = {
	id: '0xvaultid',
	token: {
		address: '0xtokenaddress',
		symbol: 'TEST'
	}
} as SgVault;

const mockDeps: VaultDepositHandlerDependencies = {
	vault: mockVault,
	network: 'ethereum',
	orderbookAddress: '0xorderbook' as Hex,
	subgraphUrl: 'https://subgraph.example.com',
	chainId: 1,
	account: '0xaccount' as Hex,
	rpcUrl: 'https://rpc.example.com',
	handleDepositModal: mockHandleDepositModal,
	handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
	errToast: mockErrToast,
	manager: mockManager as unknown as TransactionManager
};

// Mock orderbook functions
vi.mock('@rainlanguage/orderbook', async (importOriginal) => {
	const original = await importOriginal<typeof import('@rainlanguage/orderbook')>();
	return {
		...original,
		getVaultApprovalCalldata: vi.fn(),
		getVaultDepositCalldata: vi.fn()
	};
});
const { getVaultApprovalCalldata, getVaultDepositCalldata } = await import(
	'@rainlanguage/orderbook'
);

describe('handleVaultDeposit', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should call handleDepositModal with correct arguments', async () => {
		await handleVaultDeposit(mockDeps);
		expect(mockHandleDepositModal).toHaveBeenCalledWith({
			open: true,
			args: {
				vault: mockVault,
				chainId: mockDeps.chainId,
				rpcUrl: mockDeps.rpcUrl,
				subgraphUrl: mockDeps.subgraphUrl,
				account: mockDeps.account
			},
			onSubmit: expect.any(Function)
		});
	});

	describe('onSubmit callback from handleDepositModal', () => {
		const mockAmount = 100n;
		const mockApprovalCalldata = '0xapprovalcalldata' as Hex;
		const mockDepositCalldata = '0xdepositcalldata' as Hex;
		const mockTxHashApproval = '0xtxhashapproval' as Hex;
		const mockTxHashDeposit = '0xtxhashdeposit' as Hex;

		beforeEach(async () => {
			await handleVaultDeposit(mockDeps);
		});

		it('should execute deposit directly if getVaultApprovalCalldata returns error', async () => {
			vi.mocked(getVaultApprovalCalldata).mockResolvedValue({
				error: { msg: 'Approval error', readableMsg: 'Approval error readable' },
				value: undefined
			});
			vi.mocked(getVaultDepositCalldata).mockResolvedValue({
				value: mockDepositCalldata,
				error: undefined
			});

			const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
			await onSubmitCall(mockAmount);

			expect(getVaultApprovalCalldata).toHaveBeenCalledWith(
				mockDeps.rpcUrl,
				mockVault,
				mockAmount.toString()
			);
			expect(mockErrToast).not.toHaveBeenCalledWith('Approval error');
			expect(getVaultDepositCalldata).toHaveBeenCalledWith(mockVault, mockAmount.toString());
			expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
			expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith({
				open: true,
				modalTitle: 'Depositing TEST',
				closeOnConfirm: false,
				args: expect.objectContaining({ calldata: mockDepositCalldata })
			});
		});

		it('should show error toast if getVaultDepositCalldata returns an error (direct deposit flow)', async () => {
			vi.mocked(getVaultApprovalCalldata).mockResolvedValue({
				error: { msg: 'Approval error', readableMsg: 'Approval error readable' },
				value: undefined
			});
			vi.mocked(getVaultDepositCalldata).mockResolvedValue({
				error: { msg: 'Deposit error', readableMsg: 'Deposit error readable' },
				value: undefined
			});

			const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
			await onSubmitCall(mockAmount);

			expect(mockErrToast).toHaveBeenCalledWith('Deposit error');
			// ensure no transaction confirmation modal for deposit is shown
			expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
		});

		it('should handle approval and then deposit if approvalCalldata is successful', async () => {
			vi.mocked(getVaultApprovalCalldata).mockResolvedValue({
				value: mockApprovalCalldata,
				error: undefined
			});
			vi.mocked(getVaultDepositCalldata).mockResolvedValue({
				value: mockDepositCalldata,
				error: undefined
			});

			const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
			await onSubmitCall(mockAmount);

			expect(getVaultApprovalCalldata).toHaveBeenCalledTimes(1);
			expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
			expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(1, {
				open: true,
				modalTitle: 'Approving TEST spend',
				closeOnConfirm: true,
				args: {
					entity: mockVault,
					toAddress: mockVault.token.address as Hex,
					chainId: mockDeps.chainId,
					onConfirm: expect.any(Function),
					calldata: mockApprovalCalldata
				}
			});

			// Simulate approval confirmation
			const onApprovalConfirmCall =
				mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
			onApprovalConfirmCall(mockTxHashApproval);

			expect(mockCreateApprovalTransaction).toHaveBeenCalledWith({
				txHash: mockTxHashApproval,
				chainId: mockDeps.chainId,
				networkKey: mockDeps.network,
				queryKey: mockVault.id,
				entity: mockVault
			});

			expect(getVaultDepositCalldata).toHaveBeenCalledWith(mockVault, mockAmount.toString());
			await waitFor(() => {
				expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);
				expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(2, {
					open: true,
					modalTitle: 'Depositing TEST',
					closeOnConfirm: false,
					args: {
						entity: mockVault,
						toAddress: mockDeps.orderbookAddress,
						chainId: mockDeps.chainId,
						onConfirm: expect.any(Function),
						calldata: mockDepositCalldata
					}
				});
			});

			const onDepositConfirmCall =
				mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
			onDepositConfirmCall(mockTxHashDeposit);

			expect(mockCreateDepositTransaction).toHaveBeenCalledWith({
				subgraphUrl: mockDeps.subgraphUrl,
				txHash: mockTxHashDeposit,
				chainId: mockDeps.chainId,
				networkKey: mockDeps.network,
				queryKey: mockVault.id,
				entity: mockVault,
				amount: mockAmount
			});
		});

		it('should show error toast if getVaultDepositCalldata fails after successful approval', async () => {
			vi.mocked(getVaultApprovalCalldata).mockResolvedValue({
				value: mockApprovalCalldata,
				error: undefined
			});
			vi.mocked(getVaultDepositCalldata).mockResolvedValue({
				error: { msg: 'Deposit error after approval', readableMsg: 'Deposit error readable' },
				value: undefined
			});

			const onSubmitCall = mockHandleDepositModal.mock.calls[0][0].onSubmit;
			await onSubmitCall(mockAmount);

			const onApprovalConfirmCall =
				mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
			await onApprovalConfirmCall(mockTxHashApproval);

			expect(mockErrToast).toHaveBeenCalledWith('Deposit error after approval');
			expect(
				mockHandleTransactionConfirmationModal.mock.calls.filter(
					(call) => call[0].args.calldata === mockDepositCalldata
				).length
			).toBe(0);
		});
	});
});
