import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render } from '@testing-library/svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import type { OrderRemoveModalProps } from '@rainlanguage/ui-components';
import { getRemoveOrderCalldata } from '@rainlanguage/orderbook';

vi.mock('@rainlanguage/orderbook', () => ({
	getRemoveOrderCalldata: vi.fn()
}));

vi.useFakeTimers();

describe('OrderRemoveModal', () => {
	const mockOrder = {
		id: '1',
		orderHash: '0x123',
		owner: '0x456'
	};

	const defaultProps = {
		open: true,
		args: {
			order: mockOrder,
			chainId: 1,
			orderbookAddress: '0x789',
			config: {},
			onRemove: vi.fn()
		}
	} as unknown as OrderRemoveModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		(getRemoveOrderCalldata as Mock).mockResolvedValue({
			value: '0x123'
		});
	});

	it('handles transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleRemoveOrderTransaction');
		render(OrderRemoveModal, defaultProps);

		await vi.runAllTimersAsync();
		expect(handleTransactionSpy).toHaveBeenCalledWith({
			chainId: 1,
			orderbookAddress: '0x789',
			config: {},
			onRemove: defaultProps.args.onRemove,
			order: {
				id: '1',
				orderHash: '0x123',
				owner: '0x456'
			},
			removeOrderCalldata: '0x123'
		});
	});

	it('calls onRemove callback after successful transaction', async () => {
		render(OrderRemoveModal, defaultProps);

		transactionStore.transactionSuccess('0x123');
		await vi.runAllTimersAsync();

		expect(defaultProps.args.onRemove).toHaveBeenCalled();
	});
});
