import { render, screen, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import LoadingWrapper from '$lib/components/LoadingWrapper.svelte';
import { isNavigating } from '$lib/stores/loading';

vi.mock('$app/navigation', () => ({
	beforeNavigate: vi.fn((cb) => setTimeout(cb, 10)), // Simulate navigation start
	afterNavigate: vi.fn((cb) => setTimeout(cb, 50)) // Simulate navigation end
}));

describe('LoadingWrapper', () => {
	it('displays progress bar on navigation start', async () => {
		render(LoadingWrapper);

		// Simulate navigation start
		await waitFor(() => {
			expect(screen.getByTestId('progressbar')).toBeInTheDocument();
		});
	});

	it('hides progress bar after navigation ends', async () => {
		render(LoadingWrapper);

		// Wait for navigation to finish
		await waitFor(() => {
			expect(get(isNavigating)).toBe(false);
		});

		// Ensure progress bar disappears
		await waitFor(
			() => {
				const progressBar = screen.queryByTestId('progressbar');
				expect(progressBar).not.toBeInTheDocument();
			},
			{ timeout: 600 }
		);
	});

	it('does not show progress bar if no navigation occurs', () => {
		render(LoadingWrapper);

		// Progress bar should not exist initially
		const progressBar = screen.queryByTestId('progressbar');
		expect(progressBar).not.toBeInTheDocument();
	});
});
