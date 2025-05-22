import { describe, it, expect, vi, beforeEach } from 'vitest';
import { handleAddOrder } from '../lib/services/handleAddOrder';
import type { HandleAddOrderDependencies } from '../lib/services/handleAddOrder';
import type { DeploymentTransactionArgs } from '@rainlanguage/orderbook';
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

describe('handleAddOrder', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockHandleTransactionConfirmationModal.mockImplementation(
			async (props: TransactionConfirmationProps) => {
				if (props.args && typeof props.args.onConfirm === 'function') {
					await props.args.onConfirm('0xmocktxhash' as Hex);
				}
			}
		);
	});

	const baseDeploymentArgs: DeploymentTransactionArgs = {
		approvals: [],
		deploymentCalldata: '0xdeploymentCalldata' as Hex,
		orderbookAddress: '0xorderbookAddressFromArgs' as Hex,
		chainId: 1
	};

	const baseDeps: Omit<HandleAddOrderDependencies, 'args'> = {
		handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
		errToast: mockErrToast,
		manager: mockManager,
		network: 'testNetwork',
		orderbookAddress: '0xorderbookAddressFromDeps' as Hex,
		subgraphUrl: 'https://test.subgraph.com',
		chainId: 1
	};

	it('should handle an order with no approvals', async () => {
		const deps: HandleAddOrderDependencies = {
			...baseDeps,
			args: {
				...baseDeploymentArgs,
				approvals: []
			}
		};

		await handleAddOrder(deps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				open: true,
				args: expect.objectContaining({
					toAddress: deps.args.orderbookAddress,
					chainId: deps.args.chainId,
					calldata: deps.args.deploymentCalldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		expect(mockCreateAddOrderTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				...deps.args,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				networkKey: deps.network,
				subgraphUrl: deps.subgraphUrl
			})
		);
	});

	it('should handle an order with one approval', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const deps: HandleAddOrderDependencies = {
			...baseDeps,
			args: {
				...baseDeploymentArgs,
				approvals: [approval1]
			}
		};

		await handleAddOrder(deps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				open: true,
				args: expect.objectContaining({
					toAddress: approval1.token,
					chainId: deps.chainId,
					calldata: approval1.calldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		expect(mockCreateApprovalTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateApprovalTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				...deps.args,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				networkKey: deps.network

			})
		);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				open: true,
				args: expect.objectContaining({
					toAddress: deps.args.orderbookAddress,
					chainId: deps.args.chainId,
					calldata: deps.args.deploymentCalldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		expect(mockCreateAddOrderTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				...deps.args,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				networkKey: deps.network,
				subgraphUrl: deps.subgraphUrl
			})
		);
	});

	it('should handle an order with multiple approvals', async () => {
		const approval1 = { token: '0xtoken1' as Hex, calldata: '0xapprovalcalldata1' as Hex, symbol: 'TKN1' };
		const approval2 = { token: '0xtoken2' as Hex, calldata: '0xapprovalcalldata2' as Hex, symbol: 'TKN2' };
		const deps: HandleAddOrderDependencies = {
			...baseDeps,
			args: {
				...baseDeploymentArgs,
				approvals: [approval1, approval2]
			}
		};

		await handleAddOrder(deps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(3);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				args: expect.objectContaining({
					toAddress: approval1.token,
					calldata: approval1.calldata
				})
			})
		);
		expect(mockCreateApprovalTransaction).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				...deps.args,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				networkKey: deps.network
			})
		);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				args: expect.objectContaining({
					toAddress: approval2.token,
					calldata: approval2.calldata
				})
			})
		);
		expect(mockCreateApprovalTransaction).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				...deps.args,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				networkKey: deps.network
			})
		);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			3,
			expect.objectContaining({
				args: expect.objectContaining({
					toAddress: deps.args.orderbookAddress,
					chainId: deps.args.chainId,
					calldata: deps.args.deploymentCalldata
				})
			})
		);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				...deps.args,
				txHash: '0xmocktxhash',
				queryKey: QKEY_ORDERS,
				subgraphUrl: deps.subgraphUrl,
				networkKey: deps.network
			})
		);
	});

	it('should use different txHashes from modal confirmations if they differ', async () => {
		const approval1 = { token: '0xtoken1' as Hex, calldata: '0xapprovalcalldata1' as Hex, symbol: 'TKN1' };
		const deps: HandleAddOrderDependencies = {
			...baseDeps,
			args: {
				...baseDeploymentArgs,
				approvals: [approval1]
			}
		};

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

		await handleAddOrder(deps);

		expect(mockCreateApprovalTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				txHash: '0xapprovalhash'
			})
		);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				txHash: '0xaddorderhash'
			})
		);
	});
}); 