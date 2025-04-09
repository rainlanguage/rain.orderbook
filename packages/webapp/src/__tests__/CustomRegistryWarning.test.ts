import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import CustomRegistryWarning from '$lib/components/CustomRegistryWarning.svelte';
import RegistryManager from '$lib/services/RegistryManager';
import type { Mock } from 'vitest';

vi.mock('$lib/services/RegistryManager', () => ({
	default: {
		clearFromStorage: vi.fn(),
		getFromStorage: vi.fn(),
		setToStorage: vi.fn()
	}
}));

describe('CustomRegistryWarning Component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		(RegistryManager.getFromStorage as Mock).mockReturnValue('https://custom-registry.com');
		(RegistryManager.setToStorage as Mock).mockReturnValue(undefined);
	});

	it('should render the warning message correctly', () => {
		render(CustomRegistryWarning);

		const warningElement = screen.getByTestId('custom-registry-warning');
		expect(warningElement).toBeInTheDocument();

		expect(screen.getByText(/You are using a/i)).toBeInTheDocument();
		expect(screen.getByText(/custom strategies registry./i)).toBeInTheDocument();

		const defaultLink = screen.getByText('Use default.');
		expect(defaultLink).toBeInTheDocument();
		expect(defaultLink.tagName.toLowerCase()).toBe('a');
		expect(defaultLink).toHaveAttribute('href', '/deploy');
		expect(defaultLink).toHaveAttribute('data-sveltekit-reload');
	});

	it('should call clearFromStorage when "Use default" is clicked', async () => {
		render(CustomRegistryWarning);

		const defaultLink = screen.getByText('Use default.');
		await fireEvent.click(defaultLink);

		expect(RegistryManager.clearFromStorage).toHaveBeenCalledTimes(1);
	});
});
