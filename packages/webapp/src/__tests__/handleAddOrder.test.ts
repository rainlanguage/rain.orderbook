import { describe, it, expect, vi, beforeEach } from 'vitest';
import { handleAddOrder } from '../lib/services/handleAddOrder';
import type { HandleAddOrderDependencies } from '../lib/services/handleAddOrder';
import type { DeploymentTransactionArgs, DotrainOrderGui } from '@rainlanguage/orderbook';
import type { TransactionManager } from '@rainlanguage/ui-components';
import { QKEY_ORDERS } from '@rainlanguage/ui-components';
import type { Hex } from 'viem';

// Mocks
const mockHandleTransactionConfirmationModal = vi.fn().mockResolvedValue({ success: true });
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

const MOCKED_NETWORK_KEY = 'testNetwork';
const MOCKED_ACCOUNT = '0xmockAccount' as Hex;

const mockGui = {
	getDeploymentTransactionArgs: mockGetDeploymentTransactionArgs,
	getNetworkKey: mockGetNetworkKey
} as unknown as DotrainOrderGui;

const mockDeps: HandleAddOrderDependencies = {
	handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
	errToast: mockErrToast,
	manager: mockManager,
	gui: mockGui,
	account: MOCKED_ACCOUNT,
	subgraphUrl: 'https://test.subgraph.com'
};

const mockDeploymentArgs: DeploymentTransactionArgs = {
	approvals: [],
	deploymentCalldata: '0xdeploymentCalldata' as Hex,
	orderbookAddress: '0xorderbookAddressFromArgs' as Hex,
	chainId: 1
};

describe('handleAddOrder', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		// Reset to a simple mock, specific onConfirm logic will be handled in tests
		mockHandleTransactionConfirmationModal.mockReset();
		// Default to success for sequential calls
		mockHandleTransactionConfirmationModal.mockResolvedValue({ success: true });
		mockGetNetworkKey.mockReturnValue({ value: MOCKED_NETWORK_KEY, error: null });
		// mockGetDeploymentTransactionArgs will be reset/set in each test
	});

	it('should handle an order with no approvals, calling createAddOrderTransaction on confirm', async () => {
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			approvals: []
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		await handleAddOrder(mockDeps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				open: true,
				modalTitle: 'Deploying your order',
				args: expect.objectContaining({
					toAddress: currentTestSpecificArgs.orderbookAddress,
					chainId: currentTestSpecificArgs.chainId,
					calldata: currentTestSpecificArgs.deploymentCalldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		// Manually call the onConfirm for add order
		const addOrderOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		const mockAddOrderTxHash = '0xaddOrderHashFromTest' as Hex;
		await addOrderOnConfirm(mockAddOrderTxHash);

		expect(mockCreateAddOrderTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: mockAddOrderTxHash, // Check for the specific hash
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY,
				subgraphUrl: mockDeps.subgraphUrl
			})
		);
	});

	it('should handle an order with one approval, then add order, calling respective transaction creations on confirm', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			approvals: [approval1]
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		// Mock sequential calls - approval first, then deployment
		mockHandleTransactionConfirmationModal
			.mockResolvedValueOnce({ success: true, hash: '0xapprovalHash' })
			.mockResolvedValueOnce({ success: true, hash: '0xdeploymentHash' });

		await handleAddOrder(mockDeps);

		// Should be called twice: once for approval, once for deployment
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);

		// First call should be for approval
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				open: true,
				modalTitle: 'Approving token spend',
				closeOnConfirm: true,
				args: expect.objectContaining({
					toAddress: approval1.token,
					chainId: currentTestSpecificArgs.chainId,
					calldata: approval1.calldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		// Second call should be for deployment
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				open: true,
				modalTitle: 'Deploying your order',
				args: expect.objectContaining({
					toAddress: currentTestSpecificArgs.orderbookAddress,
					chainId: currentTestSpecificArgs.chainId,
					calldata: currentTestSpecificArgs.deploymentCalldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		// Verify onConfirm functions would call the right transaction managers
		const approvalOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		const deploymentOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;

		// Simulate calling onConfirm for approval
		await approvalOnConfirm('0xapprovalTxHash' as Hex);
		expect(mockCreateApprovalTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xapprovalTxHash',
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY
			})
		);

		// Simulate calling onConfirm for deployment
		await deploymentOnConfirm('0xdeploymentTxHash' as Hex);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: '0xdeploymentTxHash',
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY,
				subgraphUrl: mockDeps.subgraphUrl
			})
		);
	});

	it('should handle an order with multiple approvals, then add order, calling respective transaction creations on confirm', async () => {
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
			...mockDeploymentArgs,
			approvals: [approval1, approval2]
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		// Mock sequential calls - 2 approvals + deployment
		mockHandleTransactionConfirmationModal
			.mockResolvedValueOnce({ success: true, hash: '0xapproval1Hash' })
			.mockResolvedValueOnce({ success: true, hash: '0xapproval2Hash' })
			.mockResolvedValueOnce({ success: true, hash: '0xdeploymentHash' });

		await handleAddOrder(mockDeps);

		// Should be called 3 times: 2 approvals + 1 deployment
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(3);

		// First approval
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				modalTitle: 'Approving token spend',
				args: expect.objectContaining({ toAddress: approval1.token })
			})
		);

		// Second approval
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				modalTitle: 'Approving token spend',
				args: expect.objectContaining({ toAddress: approval2.token })
			})
		);

		// Deployment
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			3,
			expect.objectContaining({
				modalTitle: 'Deploying your order',
				args: expect.objectContaining({ toAddress: currentTestSpecificArgs.orderbookAddress })
			})
		);

		// Verify onConfirm functions would call the right transaction managers
		const approval1OnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		const approval2OnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
		const deploymentOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[2][0].args.onConfirm;

		// Simulate calling onConfirm for first approval
		await approval1OnConfirm('0xapproval1TxHash' as Hex);
		expect(mockCreateApprovalTransaction).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({ txHash: '0xapproval1TxHash', networkKey: MOCKED_NETWORK_KEY })
		);

		// Simulate calling onConfirm for second approval
		await approval2OnConfirm('0xapproval2TxHash' as Hex);
		expect(mockCreateApprovalTransaction).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({ txHash: '0xapproval2TxHash', networkKey: MOCKED_NETWORK_KEY })
		);

		// Simulate calling onConfirm for deployment
		await deploymentOnConfirm('0xdeploymentTxHash' as Hex);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				txHash: '0xdeploymentTxHash',
				networkKey: MOCKED_NETWORK_KEY,
				subgraphUrl: mockDeps.subgraphUrl
			})
		);
	});

	it('should use different txHashes from modal onConfirms when they differ', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			approvals: [approval1]
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		// Mock sequential calls with specific hashes
		mockHandleTransactionConfirmationModal
			.mockResolvedValueOnce({ success: true, hash: '0xspecificApprovalHash' })
			.mockResolvedValueOnce({ success: true, hash: '0xspecificAddOrderHash' });

		await handleAddOrder(mockDeps);

		// Should be called twice: approval + deployment
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);

		// Verify onConfirm functions would call the right transaction managers with specific hashes
		const approvalOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		const addOrderOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;

		// Simulate calling onConfirm for approval with specific hash
		const specificApprovalHash = '0xspecificApprovalHash' as Hex;
		await approvalOnConfirm(specificApprovalHash);

		expect(mockCreateApprovalTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: specificApprovalHash, // Check specific hash
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY
			})
		);

		// Simulate calling onConfirm for deployment with specific hash
		const specificAddOrderHash = '0xspecificAddOrderHash' as Hex;
		await addOrderOnConfirm(specificAddOrderHash);

		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: specificAddOrderHash, // Check specific hash
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY,
				subgraphUrl: mockDeps.subgraphUrl
			})
		);
	});

	it('should call errToast if getNetworkKey returns an error', async () => {
		mockGetNetworkKey.mockReturnValue({ value: null, error: true });

		await handleAddOrder(mockDeps);

		expect(mockErrToast).toHaveBeenCalledWith('Could not deploy: Error getting network key');
		expect(mockGetDeploymentTransactionArgs).not.toHaveBeenCalled();
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});

	it('should call errToast if account is null', async () => {
		const depsWithNullAccount = { ...mockDeps, account: null };
		mockGetNetworkKey.mockReturnValue({ value: MOCKED_NETWORK_KEY, error: null });

		await handleAddOrder(depsWithNullAccount);

		expect(mockErrToast).toHaveBeenCalledWith('Could not deploy: No wallet address found');
		expect(mockGetNetworkKey).toHaveBeenCalled();
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

		await handleAddOrder(mockDeps);

		expect(mockErrToast).toHaveBeenCalledWith(`Could not deploy: ${customErrorMsg}`);
		expect(mockGetNetworkKey).toHaveBeenCalled();
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});

	it('should call errToast and stop if approval fails', async () => {
		const approval1 = {
			token: '0xtoken1' as Hex,
			calldata: '0xapprovalcalldata1' as Hex,
			symbol: 'TKN1'
		};
		const currentTestSpecificArgs: DeploymentTransactionArgs = {
			...mockDeploymentArgs,
			approvals: [approval1]
		};
		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: currentTestSpecificArgs,
			error: null
		});

		// Mock approval failure - first call fails, second should not happen
		mockHandleTransactionConfirmationModal.mockResolvedValueOnce({ success: false }); // Approval fails

		await handleAddOrder(mockDeps);

		// Should only be called once for the failed approval
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(1);
		expect(mockErrToast).toHaveBeenCalledWith('Approval was cancelled or failed');
		expect(mockCreateApprovalTransaction).not.toHaveBeenCalled();
		expect(mockCreateAddOrderTransaction).not.toHaveBeenCalled();
	});
});
