import { describe, expect, it, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import Layout from './+layout.svelte';
import RegistryManager from '$lib/services/RegistryManager';
import type { Mock } from 'vitest';
import { loadRegistryUrl } from '$lib/services/loadRegistryUrl';

const { mockPageStore } = await vi.hoisted(() => import('$lib/__mocks__/stores'));

vi.mock('$app/stores', () => {
	return { page: mockPageStore };
});

vi.mock('$lib/services/RegistryManager', () => ({
	default: {
		setToStorage: vi.fn(),
		clearFromStorage: vi.fn(),
		isCustomRegistry: vi.fn(),
		getFromStorage: vi.fn(),
		updateUrlParam: vi.fn()
	}
}));

vi.mock('$lib/services/loadRegistryUrl', () => ({
	loadRegistryUrl: vi.fn()
}));

describe('Layout Component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should show custom registry warning when using non-default registry', () => {
		(RegistryManager.getFromStorage as Mock).mockReturnValue('https://custom-registry.com');
		(RegistryManager.isCustomRegistry as Mock).mockReturnValue(true);

		render(Layout);

		expect(RegistryManager.getFromStorage).toHaveBeenCalled();
		expect(screen.getByTestId('custom-registry-warning')).toBeTruthy();
	});

	it('should display advanced mode components when advanced mode is on', () => {
		(RegistryManager.getFromStorage as Mock).mockReturnValue('https://custom-registry.com');

		render(Layout);

		expect(screen.getByTestId('registry-input')).toBeTruthy();
	});

	it('should not display advanced mode components when advanced mode is off', () => {
		(RegistryManager.getFromStorage as Mock).mockReturnValue('');

		render(Layout);

		expect(screen.queryByTestId('registry-input')).toBeNull();
	});

	it('should handle registry URL loading with error handling', async () => {
		const errorMessage = 'Failed to update registry URL';
		(loadRegistryUrl as Mock).mockRejectedValue(new Error(errorMessage));
		(RegistryManager.getFromStorage as Mock).mockReturnValue('https://custom-registry.com');

		render(Layout);

		const registryInput = screen.getByTestId('registry-input');
		const input = registryInput.querySelector('input');
		const submitButton = registryInput.querySelector('button');

		if (input && submitButton) {
			await fireEvent.input(input, { target: { value: 'https://test.registry.url' } });
			await fireEvent.click(submitButton);

			const errorElement = await screen.findByText(errorMessage);
			expect(errorElement).toBeInTheDocument();
		}
	});

	it('should clear error before loading new registry URL', async () => {
		(RegistryManager.getFromStorage as Mock).mockReturnValue('https://custom-registry.com');

		render(Layout);

		const registryInput = screen.getByTestId('registry-input');
		const input = registryInput.querySelector('input');
		const submitButton = registryInput.querySelector('button');

		if (input && submitButton) {
			await fireEvent.input(input, { target: { value: 'https://test.registry.url' } });
			await fireEvent.click(submitButton);

			expect(loadRegistryUrl).toHaveBeenCalledWith('https://test.registry.url');
		}
	});
	it('should handle localStorage errors during initialization', async () => {
		(RegistryManager.getFromStorage as Mock).mockImplementation(() => {
			throw new Error('localStorage access error');
		});

		render(Layout);

		await waitFor(() => {
			const errorElement = screen.getByTestId('registry-error');
			expect(errorElement).toHaveTextContent('localStorage access error');
		});
	});
});
