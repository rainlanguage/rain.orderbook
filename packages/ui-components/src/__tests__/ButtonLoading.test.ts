import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ButtonLoading from '../lib/components/ButtonLoading.svelte';

describe('ButtonLoading', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders with default props', () => {
		const { container } = render(ButtonLoading, {
			props: {
				loading: false,
				disabled: false
			}
		});

		expect(container).toBeTruthy();
		const button = screen.getByRole('button');
		expect(button).toBeInTheDocument();

		// Spinner should not be visible when not loading
		expect(screen.queryByTestId('spinner')).not.toBeInTheDocument();
	});

	it('renders the spinner when loading is true', () => {
		render(ButtonLoading, {
			props: {
				loading: true
			}
		});

		expect(screen.getByTestId('spinner-element')).toBeInTheDocument();
	});

	it('disables the button when loading is true', () => {
		render(ButtonLoading, {
			props: {
				loading: true
			}
		});

		const button = screen.getByRole('button');
		expect(button).toHaveAttribute('disabled');
	});

	it('disables the button when disabled is true', () => {
		render(ButtonLoading, {
			props: {
				disabled: true
			}
		});

		const button = screen.getByRole('button');
		expect(button).toHaveAttribute('disabled');
	});

	it('forwards additional attributes to Button component', () => {
		render(ButtonLoading, {
			props: {
				class: 'custom-class',
				'data-testid': 'custom-button'
			}
		});

		const button = screen.getByTestId('custom-button');
		expect(button).toBeInTheDocument();
		expect(button).toHaveClass('custom-class');
	});

	it('emits click events when not disabled', async () => {
		const { component } = render(ButtonLoading);

		const mockClickHandler = vi.fn();
		component.$on('click', mockClickHandler);

		const button = screen.getByRole('button');
		await fireEvent.click(button);

		expect(mockClickHandler).toHaveBeenCalled();
	});

	it('conditionally renders spinner based on loading state changes', async () => {
		const { component } = render(ButtonLoading, {
			props: {
				loading: false
			}
		});

		// Initially no spinner
		expect(screen.queryByTestId('spinner-element')).not.toBeInTheDocument();

		// Update the loading prop
		await component.$set({ loading: true });

		// Now the spinner should be visible
		expect(screen.getByTestId('spinner-element')).toBeInTheDocument();

		// Update loading back to false
		await component.$set({ loading: false });

		// Spinner should be gone again
		expect(screen.queryByTestId('spinner-element')).not.toBeInTheDocument();
	});
});
