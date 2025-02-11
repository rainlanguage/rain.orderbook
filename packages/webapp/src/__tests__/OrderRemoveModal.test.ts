import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import type { ComponentProps } from 'svelte';

export type ModalProps = ComponentProps<OrderRemoveModal>;

vi.mock('@rainlanguage/orderbook/js_api', () => ({
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
		order: mockOrder,
		onRemove: vi.fn(),
		wagmiConfig: {},
		chainId: 1,
		orderbookAddress: '0x789'
	} as unknown as ModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		transactionStore.reset();
	});

	it('handles transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleRemoveOrderTransaction');
		render(OrderRemoveModal, defaultProps);

		await vi.runAllTimersAsync();

		expect(handleTransactionSpy).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: 1,
				orderbookAddress: '0x789',
				config: {}
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

		expect(defaultProps.onRemove).toHaveBeenCalled();
	});
});
