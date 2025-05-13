import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import type { OrderRemoveModalProps } from '@rainlanguage/ui-components';

vi.mock('@rainlanguage/orderbook', () => ({
	getRemoveOrderCalldata: vi.fn().mockResolvedValue('0x123')
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
	});

	it('handles transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleRemoveOrderTransaction');
		render(OrderRemoveModal, defaultProps);

		await vi.runAllTimersAsync();
		expect(handleTransactionSpy).toHaveBeenCalledWith({
			chainId: 1,
			orderbookAddress: '0x789',
			config: {},
			onRemove: expect.any(Function),
			order: {
				id: '1',
				orderHash: '0x123',
				owner: '0x456'
			},
			removeOrderCalldata: undefined
		});
	});

	it('calls onRemove callback after successful transaction', async () => {
		render(OrderRemoveModal, defaultProps);

		transactionStore.transactionSuccess('0x123');
		await vi.runAllTimersAsync();

		expect(defaultProps.args.onRemove).toHaveBeenCalled();
	});
});
