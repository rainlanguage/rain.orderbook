import { describe, it, expect, vi, beforeEach } from 'vitest';
import { handleAddOrder } from '../lib/services/handleAddOrder';
import type { HandleAddOrderDependencies } from '../lib/services/handleAddOrder';
import type { DeploymentTransactionArgs, DotrainOrderGui } from '@rainlanguage/orderbook';
import type { TransactionManager, TransactionConfirmationProps } from '@rainlanguage/ui-components';
import { QKEY_ORDERS } from '@rainlanguage/ui-components';
import type { Hex } from 'viem';

// Mocks
const mockHandleTransactionConfirmationModal = vi.fn();
const mockErrToast = vi.fn();
const mockCreateApprovalTransaction = vi.fn();
const mockCreateAddOrderTransaction = vi.fn();

const mockManager = {
	createApprovalTransaction: mockCreateApprovalTransaction,
	createAddOrderTransaction: mockCreateAddOrderTransaction
} as unknown as TransactionManager;

// New Mocks for gui
const mockGetDeploymentTransactionArgs = vi.fn();
const mockGetNetworkKey = vi.fn();
const mockGui = {
	getDeploymentTransactionArgs: mockGetDeploymentTransactionArgs,
	getNetworkKey: mockGetNetworkKey,
} as unknown as DotrainOrderGui;

const MOCKED_NETWORK_KEY = 'testNetwork';
const MOCKED_ACCOUNT = '0xmockAccount' as Hex;

describe('handleAddOrder', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockHandleTransactionConfirmationModal.mockImplementation(
			async (props: TransactionConfirmationProps) => {
				if (props.args && typeof props.args.onConfirm === 'function') {
					// Default tx hash for most confirmations
					await props.args.onConfirm('0xmocktxhash' as Hex);
				}
			}
		);
		mockGetNetworkKey.mockReturnValue({ value: MOCKED_NETWORK_KEY, error: null });
		// mockGetDeploymentTransactionArgs will be reset/set in each test
	});

	const baseDeploymentArgs: DeploymentTransactionArgs = {
		approvals: [], // This will be overridden in specific tests or used as is
		deploymentCalldata: '0xdeploymentCalldata' as Hex,
		orderbookAddress: '0xorderbookAddressFromArgs' as Hex,
		chainId: 1
	};

	// Common dependencies for handleAddOrder
	const commonTestDeps: HandleAddOrderDependencies = {
		handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
		errToast: mockErrToast,
		manager: mockManager,
		gui: mockGui,
		account: MOCKED_ACCOUNT,
		subgraphUrl: 'https://test.subgraph.com',
	};

	it('should handle an order with no approvals', async () => {
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...baseDeploymentArgs,
			approvals: [] // Explicitly no approvals for this test
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		await handleAddOrder(commonTestDeps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				open: true,
				args: expect.objectContaining({
					toAddress: currentTestSpecificArgs.orderbookAddress,
					chainId: currentTestSpecificArgs.chainId,
					calldata: currentTestSpecificArgs.deploymentCalldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		expect(mockCreateAddOrderTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY,
				subgraphUrl: commonTestDeps.subgraphUrl
			})
		);
	});

	it('should handle an order with one approval', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...baseDeploymentArgs,
			approvals: [approval1]
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		await handleAddOrder(commonTestDeps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				open: true,
				args: expect.objectContaining({
					toAddress: approval1.token,
					chainId: currentTestSpecificArgs.chainId,
					calldata: approval1.calldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		expect(mockCreateApprovalTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateApprovalTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY
			})
		);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				open: true,
				args: expect.objectContaining({
					toAddress: currentTestSpecificArgs.orderbookAddress,
					chainId: currentTestSpecificArgs.chainId,
					calldata: currentTestSpecificArgs.deploymentCalldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		expect(mockCreateAddOrderTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY,
				subgraphUrl: commonTestDeps.subgraphUrl
			})
		);
	});

	it('should handle an order with multiple approvals', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const approval2 = {
			token: '0xtoken2' as Hex,
			calldata: '0xapprovalcalldata2' as Hex,
			symbol: 'TKN2'
		};
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...baseDeploymentArgs,
			approvals: [approval1, approval2]
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		await handleAddOrder(commonTestDeps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(3);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				open: true,
				args: expect.objectContaining({
					toAddress: approval1.token,
					chainId: currentTestSpecificArgs.chainId,
					calldata: approval1.calldata,
					onConfirm: expect.any(Function)
				})
			})
		);
		expect(mockCreateApprovalTransaction).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY
			})
		);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				open: true,
				args: expect.objectContaining({
					toAddress: approval2.token,
					chainId: currentTestSpecificArgs.chainId,
					calldata: approval2.calldata,
					onConfirm: expect.any(Function)
				})
			})
		);
		expect(mockCreateApprovalTransaction).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY
			})
		);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			3,
			expect.objectContaining({
				open: true,
				args: expect.objectContaining({
					toAddress: currentTestSpecificArgs.orderbookAddress,
					chainId: currentTestSpecificArgs.chainId,
					calldata: currentTestSpecificArgs.deploymentCalldata,
					onConfirm: expect.any(Function)
				})
			})
		);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				subgraphUrl: commonTestDeps.subgraphUrl,
				networkKey: MOCKED_NETWORK_KEY
			})
		);
	});

	it('should use different txHashes from modal confirmations if they differ', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...baseDeploymentArgs,
			approvals: [approval1]
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		mockHandleTransactionConfirmationModal
			.mockImplementationOnce(async (props: TransactionConfirmationProps) => {
				if (props.args && typeof props.args.onConfirm === 'function') {
					await props.args.onConfirm('0xapprovalhash' as Hex);
				}
			})
			.mockImplementationOnce(async (props: TransactionConfirmationProps) => {
				if (props.args && typeof props.args.onConfirm === 'function') {
					await props.args.onConfirm('0xaddorderhash' as Hex);
				}
			});

		await handleAddOrder(commonTestDeps);

		expect(mockCreateApprovalTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xapprovalhash',
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY
			})
		);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xaddorderhash',
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY,
				subgraphUrl: commonTestDeps.subgraphUrl
			})
		);
	});

	it('should call errToast if getNetworkKey returns an error', async () => {
		mockGetNetworkKey.mockReturnValue({ value: null, error: true });

		await handleAddOrder(commonTestDeps);

		expect(mockErrToast).toHaveBeenCalledWith(
			'Could not deploy: Error getting network key'
		);
		expect(mockGetDeploymentTransactionArgs).not.toHaveBeenCalled();
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});

	it('should call errToast if account is null', async () => {
		const depsWithNullAccount = { ...commonTestDeps, account: null };
		mockGetNetworkKey.mockReturnValue({ value: MOCKED_NETWORK_KEY, error: null });

		await handleAddOrder(depsWithNullAccount);

		expect(mockErrToast).toHaveBeenCalledWith(
			'Could not deploy: No wallet address found'
		);
		expect(mockGetDeploymentTransactionArgs).not.toHaveBeenCalled();
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});

	it('should call errToast if getDeploymentTransactionArgs returns an error', async () => {
		const customErrorMsg = 'Custom error from gui';
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: null,
			error: { msg: customErrorMsg }
		});
		// Ensure getNetworkKey is fine for this test
		mockGetNetworkKey.mockReturnValue({ value: MOCKED_NETWORK_KEY, error: null });

		await handleAddOrder(commonTestDeps);

		expect(mockErrToast).toHaveBeenCalledWith(
			`Could not deploy: ${customErrorMsg}`
		);
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});
});
