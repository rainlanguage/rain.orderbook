import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import type { OrderRemoveModalProps } from '@rainlanguage/ui-components';
import { getRemoveOrderCalldata } from '@rainlanguage/orderbook';

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

	beforeEach(async () => {
		transactionStore.reset();
		(getRemoveOrderCalldata as Mock).mockResolvedValue('0x123');
	});

	it.only('handles transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleRemoveOrderTransaction');
		render(OrderRemoveModal, defaultProps);

		await vi.runAllTimersAsync();

		expect(handleTransactionSpy).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: 1,
				orderbookAddress: '0x789',
				config: {},
				removeOrderCalldata: '0x123'
			})
		);
	});

	it('closes modal and resets transaction store', async () => {
		render(OrderRemoveModal, defaultProps);
		const resetSpy = vi.spyOn(transactionStore, 'reset');

		const closeButton = screen.getByLabelText('Close modal');
		await fireEvent.click(closeButton);

		expect(resetSpy).toHaveBeenCalled();
	});

	it('calls onRemove callback after successful transaction', async () => {
		render(OrderRemoveModal, defaultProps);

		transactionStore.transactionSuccess('0x123');
		await vi.runAllTimersAsync();

		expect(defaultProps.args.onRemove).toHaveBeenCalled();
	});
});
