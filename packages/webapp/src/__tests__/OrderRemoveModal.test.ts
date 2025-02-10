import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import type { ComponentProps } from 'svelte';

export type ModalProps = ComponentProps<OrderRemoveModal>;

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	getRemoveOrderCalldata: vi.fn().mockResolvedValue('0x123')
}));

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

	it('renders transaction modal correctly', () => {
		render(OrderRemoveModal, defaultProps);
		expect(screen.getByText('Removing order...')).toBeInTheDocument();
	});

	it('handles transaction correctly', () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleRemoveOrderTransaction');
		render(OrderRemoveModal, defaultProps);

		expect(handleTransactionSpy).toHaveBeenCalledWith(
			expect.objectContaining({
				chainId: 1,
				orderbookAddress: '0x789',
				config: {}
			})
		);
	});

	it('closes modal and resets transaction store', () => {
		const { component } = render(OrderRemoveModal, defaultProps);
		const resetSpy = vi.spyOn(transactionStore, 'reset');

		component.$set({ open: false });

		expect(resetSpy).toHaveBeenCalled();
	});

	it('calls onRemove callback after successful transaction', () => {
		render(OrderRemoveModal, defaultProps);
		transactionStore.transactionSuccess('0x123');

		expect(defaultProps.onRemove).toHaveBeenCalled();
	});
});
