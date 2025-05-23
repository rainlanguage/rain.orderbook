import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	handleVaultWithdraw,
	type VaultWithdrawHandlerDependencies
} from '../lib/services/handleVaultWithdraw';
import type { SgVault } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import type { TransactionManager } from '@rainlanguage/ui-components';

// Mocks
const mockHandleWithdrawModal = vi.fn();
const mockHandleTransactionConfirmationModal = vi.fn();
const mockErrToast = vi.fn();
const mockCreateWithdrawTransaction = vi.fn();

const mockManager = {
	createWithdrawTransaction: mockCreateWithdrawTransaction
};

const mockOnSubmit = vi.fn();

const mockVault = {
	id: '0xvaultid',
	token: {
		symbol: 'TEST'
	}
} as SgVault;

const mockDeps: VaultWithdrawHandlerDependencies = {
	vault: mockVault,
	network: 'ethereum',
	orderbookAddress: '0xorderbook' as Hex,
	subgraphUrl: 'https://subgraph.example.com',
	chainId: 1,
	account: '0xaccount' as Hex,
	rpcUrl: 'https://rpc.example.com',
	handleWithdrawModal: mockHandleWithdrawModal,
	handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
	errToast: mockErrToast,
	manager: mockManager as unknown as TransactionManager
};

// Mock getVaultWithdrawCalldata
vi.mock('@rainlanguage/orderbook', async (importOriginal) => {
	const original = await importOriginal<typeof import('@rainlanguage/orderbook')>();
	return {
		...original,
		getVaultWithdrawCalldata: vi.fn()
	};
});
const { getVaultWithdrawCalldata } = await import('@rainlanguage/orderbook');

describe('handleVaultWithdraw', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should call handleWithdrawModal with correct arguments', async () => {
		await handleVaultWithdraw(mockDeps);
		expect(mockHandleWithdrawModal).toHaveBeenCalledWith({
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

	it('should show error toast if getVaultWithdrawCalldata returns an error', async () => {
		vi.mocked(getVaultWithdrawCalldata).mockResolvedValue({
			error: { msg: 'Calldata error', readableMsg: 'Calldata error readable' },
			value: undefined
		});

		await handleVaultWithdraw(mockDeps);

		const onSubmitCall = mockHandleWithdrawModal.mock.calls[0][0].onSubmit;
		await onSubmitCall(100n);

		expect(mockErrToast).toHaveBeenCalledWith('Calldata error');
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
	});

	it('should show error toast if getVaultWithdrawCalldata throws', async () => {
		vi.mocked(getVaultWithdrawCalldata).mockRejectedValue(new Error('Fetch failed'));

		await handleVaultWithdraw(mockDeps);

		const onSubmitCall = mockHandleWithdrawModal.mock.calls[0][0].onSubmit;
		await onSubmitCall(100n);

		expect(mockErrToast).toHaveBeenCalledWith('Failed to get calldata for vault withdrawal.');
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
	});

	it('should call handleTransactionConfirmationModal on successful calldata fetch', async () => {
		const mockCalldata = '0xcalldata' as Hex;
		vi.mocked(getVaultWithdrawCalldata).mockResolvedValue({
			value: mockCalldata,
			error: undefined
		});

		await handleVaultWithdraw(mockDeps);

		const onSubmitCall = mockHandleWithdrawModal.mock.calls[0][0].onSubmit;
		await onSubmitCall(100n);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith({
			open: true,
			modalTitle: 'Withdrawing 100 TEST...',
			args: {
				entity: mockVault,
				orderbookAddress: mockDeps.orderbookAddress,
				chainId: mockDeps.chainId,
				onConfirm: expect.any(Function),
				calldata: mockCalldata
			}
		});
		expect(mockErrToast).not.toHaveBeenCalled();
	});

	it('should call manager.createWithdrawTransaction on transaction confirmation', async () => {
		const mockCalldata = '0xcalldata' as Hex;
		const mockTxHash = '0xtxhash' as Hex;
		vi.mocked(getVaultWithdrawCalldata).mockResolvedValue({
			value: mockCalldata,
			error: undefined
		});

		await handleVaultWithdraw(mockDeps);

		const onSubmitCall = mockHandleWithdrawModal.mock.calls[0][0].onSubmit;
		await onSubmitCall(100n);

		const onConfirmCall = mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		onConfirmCall(mockTxHash);

		expect(mockCreateWithdrawTransaction).toHaveBeenCalledWith({
			subgraphUrl: mockDeps.subgraphUrl,
			txHash: mockTxHash,
			chainId: mockDeps.chainId,
			networkKey: mockDeps.network,
			queryKey: mockVault.id
		});
	});
});
