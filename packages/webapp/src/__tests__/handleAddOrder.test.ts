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
				modalTitle: 'Deploying your strategy',
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

		await handleAddOrder(mockDeps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2); // Initially called for approval

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				open: true,
				modalTitle: 'Approving token spend',
				args: expect.objectContaining({
					toAddress: approval1.token,
					chainId: currentTestSpecificArgs.chainId,
					calldata: approval1.calldata,
					onConfirm: expect.any(Function)
				})
			})
		);

		// Manually call the onConfirm for approval
		const approvalOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		const mockApprovalTxHash = '0xapprovalHashFromTest' as Hex;
		await approvalOnConfirm(mockApprovalTxHash);

		expect(mockCreateApprovalTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateApprovalTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: currentTestSpecificArgs.chainId,
				txHash: mockApprovalTxHash, // Check for the specific hash
				queryKey: QKEY_ORDERS,
				networkKey: MOCKED_NETWORK_KEY
			})
		);

		// Expect modal to be called again for the add order transaction
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({
				open: true,
				modalTitle: 'Deploying your strategy',
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
			mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
		const mockAddOrderTxHash = '0xaddOrderHashFromTest2' as Hex;
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

		await handleAddOrder(mockDeps);

		// First Approval
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(3);
		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({ args: expect.objectContaining({ toAddress: approval1.token }) })
		);
		const approval1OnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		const mockApproval1TxHash = '0xapproval1Hash' as Hex;
		await approval1OnConfirm(mockApproval1TxHash);

		expect(mockCreateApprovalTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateApprovalTransaction).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({ txHash: mockApproval1TxHash, networkKey: MOCKED_NETWORK_KEY })
		);

		// Second Approval

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({ args: expect.objectContaining({ toAddress: approval2.token }) })
		);
		const approval2OnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
		const mockApproval2TxHash = '0xapproval2Hash' as Hex;
		await approval2OnConfirm(mockApproval2TxHash);

		expect(mockCreateApprovalTransaction).toHaveBeenCalledTimes(2);
		expect(mockCreateApprovalTransaction).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({ txHash: mockApproval2TxHash, networkKey: MOCKED_NETWORK_KEY })
		);

		// Add Order

		expect(mockHandleTransactionConfirmationModal).toHaveBeenNthCalledWith(
			3,
			expect.objectContaining({
				args: expect.objectContaining({ toAddress: currentTestSpecificArgs.orderbookAddress })
			})
		);
		const addOrderOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[2][0].args.onConfirm;
		const mockAddOrderTxHash = '0xaddOrderFinalHash' as Hex;
		await addOrderOnConfirm(mockAddOrderTxHash);

		expect(mockCreateAddOrderTransaction).toHaveBeenCalledTimes(1);
		expect(mockCreateAddOrderTransaction).toHaveBeenCalledWith(
			expect.objectContaining({
				txHash: mockAddOrderTxHash,
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

		// No mockImplementationOnce here, we'll control the hash via onConfirm calls
		await handleAddOrder(mockDeps);

		// Approval Confirmation
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);
		const approvalOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
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

		// Add Order Confirmation
		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledTimes(2);
		const addOrderOnConfirm =
			mockHandleTransactionConfirmationModal.mock.calls[1][0].args.onConfirm;
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
});
