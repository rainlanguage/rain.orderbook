import { render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, expect, describe, vi, afterEach } from 'vitest';
import License from '../lib/components/License.svelte';

// Mock the global fetch function
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

vi.mock('svelte-markdown', async () => {
	const MockMarkdown = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockMarkdown };
});

describe('License', () => {
	const mockMarkdownContent = 'This is license text.';
	const expectedUrl =
		'https://raw.githubusercontent.com/rainlanguage/decentralicense/refs/heads/master/README.md';

	beforeEach(() => {
		mockFetch.mockReset();
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('fetches and renders markdown content on mount', async () => {
		mockFetch.mockResolvedValue({
			ok: true,
			text: async () => mockMarkdownContent
		});

		render(License);

		await waitFor(() => {
			expect(mockFetch).toHaveBeenCalledTimes(1);
			expect(mockFetch).toHaveBeenCalledWith(expectedUrl);
		});

		await waitFor(() => {
			expect(screen.getByTestId('mock-component').getAttribute('source')).toBe(mockMarkdownContent);
		});
	});

	it('handles fetch network error gracefully', async () => {
		mockFetch.mockRejectedValue(new Error('Network error'));

		const { container } = render(License);

		await waitFor(() => {
			expect(mockFetch).toHaveBeenCalledTimes(1);
			expect(mockFetch).toHaveBeenCalledWith(expectedUrl);
		});

		await waitFor(() => {
			expect(screen.getByTestId('mock-component').getAttribute('source')).toBe('Failed to fetch license');
		});
	});
});
