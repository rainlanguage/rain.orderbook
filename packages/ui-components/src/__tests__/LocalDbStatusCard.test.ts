import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/svelte';
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import LocalDbStatusCard from '../lib/components/LocalDbStatusCard.svelte';

describe('LocalDbStatusCard', () => {
	const originalClipboard = navigator.clipboard;

	beforeEach(() => {
		Object.defineProperty(navigator, 'clipboard', {
			value: {
				writeText: vi.fn().mockResolvedValue(undefined)
			},
			configurable: true,
			writable: true
		});
	});

	afterEach(() => {
		cleanup();
		if (originalClipboard) {
			Object.defineProperty(navigator, 'clipboard', {
				value: originalClipboard,
				configurable: true,
				writable: true
			});
		} else {
			Reflect.deleteProperty(navigator, 'clipboard');
		}
	});

	it('renders the default label and badge', () => {
		render(LocalDbStatusCard);

		expect(screen.getByText('LocalDB')).toBeInTheDocument();
		expect(screen.getByTestId('local-db-status')).toBeInTheDocument();
	});

	it('shows a copy button when failure and hides the raw error text', async () => {
		render(LocalDbStatusCard, {
			props: {
				label: 'Runner',
				status: 'failure',
				error: 'Runner error occurred'
			}
		});

		expect(screen.getByText('Runner')).toBeInTheDocument();
		expect(screen.getByText('Failure')).toBeInTheDocument();
		const copyButton = screen.getByTestId('local-db-error-copy');
		expect(copyButton).toHaveTextContent('Copy error details');
		expect(screen.queryByText('Runner error occurred')).not.toBeInTheDocument();

		await fireEvent.click(copyButton);

		await waitFor(() => {
			expect(copyButton).toHaveTextContent('Copied!');
		});
		expect(navigator.clipboard?.writeText).toHaveBeenCalledWith('Runner error occurred');
	});

	it('omits the copy button when status is not failure', () => {
		render(LocalDbStatusCard, {
			props: {
				status: 'syncing',
				error: 'Some warning'
			}
		});

		expect(screen.queryByTestId('local-db-error-copy')).toBeNull();
	});
});
