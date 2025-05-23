import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	handleRemoveOrder,
	type HandleRemoveOrderDependencies
} from '../lib/services/handleRemoveOrder'; // Assuming path is correct
import type { SgOrder } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import type { TransactionManager } from '@rainlanguage/ui-components';

// Mocks
const mockHandleTransactionConfirmationModal = vi.fn();
const mockErrToast = vi.fn();
const mockCreateRemoveOrderTransaction = vi.fn();
``;

const mockManager = {
	createRemoveOrderTransaction: mockCreateRemoveOrderTransaction
};

const mockOrder = {
	id: '0xorderid'
} as SgOrder;

const mockDeps: HandleRemoveOrderDependencies = {
	network: 'ethereum',
	orderbookAddress: '0xorderbook' as Hex,
	subgraphUrl: 'https://subgraph.example.com',
	chainId: 1,
	orderHash: '0xorderhashfromparams',
	handleTransactionConfirmationModal: mockHandleTransactionConfirmationModal,
	errToast: mockErrToast,
	manager: mockManager as unknown as TransactionManager
};

vi.mock('@rainlanguage/orderbook', async (importOriginal) => {
	const original = await importOriginal<typeof import('@rainlanguage/orderbook')>();
	return {
		...original,
		getRemoveOrderCalldata: vi.fn()
	};
});
const { getRemoveOrderCalldata } = await import('@rainlanguage/orderbook');

describe('handleRemoveOrder', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should show error toast if getRemoveOrderCalldata returns an error', async () => {
		vi.mocked(getRemoveOrderCalldata).mockResolvedValue({
			error: { msg: 'Calldata error', readableMsg: 'Calldata error readable' },
			value: undefined
		});

		await handleRemoveOrder(mockOrder, mockDeps);

		expect(mockErrToast).toHaveBeenCalledWith('Calldata error');
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
	});

	it('should show error toast if getRemoveOrderCalldata throws', async () => {
		vi.mocked(getRemoveOrderCalldata).mockRejectedValue(new Error('Fetch failed'));

		await handleRemoveOrder(mockOrder, mockDeps);

		expect(mockErrToast).toHaveBeenCalledWith('Failed to get calldata for order removal.');
		expect(mockHandleTransactionConfirmationModal).not.toHaveBeenCalled();
	});

	it('should call handleTransactionConfirmationModal on successful calldata fetch', async () => {
		const mockCalldata = '0xcalldata' as Hex;
		vi.mocked(getRemoveOrderCalldata).mockResolvedValue({
			value: mockCalldata,
			error: undefined
		});

		await handleRemoveOrder(mockOrder, mockDeps);

		expect(mockHandleTransactionConfirmationModal).toHaveBeenCalledWith({
			open: true,
			args: {
				entity: mockOrder,
				toAddress: mockDeps.orderbookAddress,
				chainId: mockDeps.chainId,
				onConfirm: expect.any(Function),
				calldata: mockCalldata
			}
		});
		expect(mockErrToast).not.toHaveBeenCalled();
	});

	it('should call manager.createRemoveOrderTransaction on transaction confirmation', async () => {
		const mockCalldata = '0xcalldata' as Hex;
		const mockTxHash = '0xtxhash' as Hex;
		vi.mocked(getRemoveOrderCalldata).mockResolvedValue({
			value: mockCalldata,
			error: undefined
		});

		await handleRemoveOrder(mockOrder, mockDeps);

		const onConfirmCall = mockHandleTransactionConfirmationModal.mock.calls[0][0].args.onConfirm;
		onConfirmCall(mockTxHash);

		expect(mockCreateRemoveOrderTransaction).toHaveBeenCalledWith({
			subgraphUrl: mockDeps.subgraphUrl,
			txHash: mockTxHash,
			queryKey: mockDeps.orderHash,
			chainId: mockDeps.chainId,
			networkKey: mockDeps.network,
			entity: mockOrder
		});
	});
});
