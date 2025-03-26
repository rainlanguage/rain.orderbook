import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import userEvent from '@testing-library/user-event';
import RemoveOrderButton from '../lib/components/actions/RemoveOrderButton.svelte';
import type { SgOrder } from '@rainlanguage/orderbook/js_api';

describe('RemoveOrderButton', () => {
	const mockOrder: SgOrder = {
		id: '1',
		orderHash: '0xabc123',
		owner: '0x123456',
		orderbook: {
			id: '0x789'
		},
		inputs: [],
		outputs: [],
		active: true,
		timestampAdded: '1679529600',
		orderBytes: '0x123',
		addEvents: [],
		trades: [],
		removeEvents: []
	};

	const defaultProps = {
		order: mockOrder,
		testId: 'remove-order-button'
	};

	beforeEach(() => {
		vi.resetAllMocks();
	});

	it('renders with default props', () => {
		render(RemoveOrderButton, defaultProps);
		const button = screen.getByTestId('remove-order-button');
		expect(button).toBeInTheDocument();
		expect(screen.getByText('Remove')).toBeInTheDocument();
	});

	it('renders with custom label', () => {
		const propsWithCustomLabel = {
			...defaultProps,
			label: 'Delete Order'
		};

		render(RemoveOrderButton, propsWithCustomLabel);
		expect(screen.getByText('Delete Order')).toBeInTheDocument();
	});

	it('applies custom class when provided', () => {
		const propsWithClass = {
			...defaultProps,
			customClass: 'test-custom-class'
		};

		render(RemoveOrderButton, propsWithClass);
		const button = screen.getByTestId('remove-order-button');
		expect(button.classList.contains('test-custom-class')).toBe(true);
	});

	it('disables the button when disabled prop is true', async () => {
		const user = userEvent.setup();
		const mockOnRemove = vi.fn();

		const disabledProps = {
			...defaultProps,
			disabled: true
		};

		const { component } = render(RemoveOrderButton, disabledProps);
		component.$on('remove', mockOnRemove);

		const button = screen.getByTestId('remove-order-button');
		expect(button).toBeDisabled();

		await user.click(button);
		expect(mockOnRemove).not.toHaveBeenCalled();
	});

	it('emits remove event with order when clicked', async () => {
		const user = userEvent.setup();
		const mockOnRemove = vi.fn();

		const { component } = render(RemoveOrderButton, defaultProps);
		component.$on('remove', mockOnRemove);

		const button = screen.getByTestId('remove-order-button');
		await user.click(button);

		expect(mockOnRemove).toHaveBeenCalled();
		expect(mockOnRemove.mock.calls[0][0].detail).toEqual(
			expect.objectContaining({
				order: mockOrder
			})
		);
	});

	it('includes onSuccess callback in event payload if provided', async () => {
		const user = userEvent.setup();
		const mockOnRemove = vi.fn();
		const mockOnSuccess = vi.fn();

		const propsWithSuccess = {
			...defaultProps,
			onSuccess: mockOnSuccess
		};

		const { component } = render(RemoveOrderButton, propsWithSuccess);
		component.$on('remove', mockOnRemove);

		const button = screen.getByTestId('remove-order-button');
		await user.click(button);

		expect(mockOnRemove).toHaveBeenCalled();

		const eventDetail = mockOnRemove.mock.calls[0][0].detail;
		expect(eventDetail.onSuccess).toBe(mockOnSuccess);

		eventDetail.onSuccess();
		expect(mockOnSuccess).toHaveBeenCalled();
	});
});
