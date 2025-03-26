import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import type { OrderRemoveModalProps } from '@rainlanguage/ui-components';

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
		transactionStore.reset();
	});

	it('handles transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleRemoveOrderTransaction');
		render(OrderRemoveModal, defaultProps);


		await waitFor(() => {
		expect(handleTransactionSpy).toHaveBeenCalledWith(
			expect.objectContaining({
					chainId: 1,
					orderbookAddress: '0x789',
					config: {},
					removeOrderCalldata: '0x123'
				})
			);
		});
	});

	it('closes modal and resets transaction store', async () => {
		render(OrderRemoveModal, defaultProps);
		const resetSpy = vi.spyOn(transactionStore, 'reset');

		const closeButton = screen.getByLabelText('Close modal');
		await fireEvent.click(closeButton);

		expect(resetSpy).toHaveBeenCalled();
	});

		it('calls onRemove callback after successful transaction', async () => {
		vi.useRealTimers();
		
		render(OrderRemoveModal, defaultProps);
		const onRemoveSpy = vi.fn();
		defaultProps.args.onRemove = onRemoveSpy;

		transactionStore.transactionSuccess('0x123');

		await new Promise((resolve) => setTimeout(resolve, 5100));

		expect(onRemoveSpy).toHaveBeenCalled();

		vi.useFakeTimers();
	});
});
