import { render, screen, waitFor } from '@testing-library/svelte';
import OrderPage from '../lib/components/deployment/OrderPage.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook';
import { vi, describe, it, expect, beforeEach, type Mock } from 'vitest';

// Mock fetch
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

vi.mock('../lib/components/deployment/DeploymentsSection.svelte', async () => {
	const MockDeploymentsSection = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockDeploymentsSection };
});

describe('OrderPage', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
	});

	it('renders order details successfully with rawDotrain', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockOrderDetails = {
			value: {
				name: 'Test Order',
				description: 'Test Description',
				short_description: 'Test Short Description'
			}
		};
		(DotrainOrderGui.getOrderDetails as Mock).mockResolvedValueOnce(mockOrderDetails);

		render(OrderPage, {
			props: {
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Order')).toBeInTheDocument();
			expect(screen.getByText('Test Description')).toBeInTheDocument();
		});
	});

	it('renders order details successfully from fetch', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockOrderDetails = {
			value: {
				name: 'Test Order',
				description: 'Test Description',
				short_description: 'Test Short Description'
			}
		};

		mockFetch.mockResolvedValueOnce({
			text: () => Promise.resolve(mockDotrain)
		});

		(DotrainOrderGui.getOrderDetails as Mock).mockResolvedValueOnce(mockOrderDetails);

		render(OrderPage, {
			props: {
				orderName: 'TestOrder',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Order')).toBeInTheDocument();
			expect(screen.getByText('Test Description')).toBeInTheDocument();
		});
	});

	it('displays error message when order details fail', async () => {
		const mockDotrain = 'mock dotrain content';

		// Mock fetch response
		mockFetch.mockResolvedValueOnce({
			text: () => Promise.resolve(mockDotrain)
		});

		// Mock DotrainOrderGui methods
		(DotrainOrderGui.getOrderDetails as Mock).mockResolvedValueOnce({
			error: {
				msg: 'Failed to get order details'
			}
		});

		render(OrderPage, {
			props: {
				orderName: 'TestOrder',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Error: Failed to get order details')).toBeInTheDocument();
		});
	});

	it('handles markdown fetch failure', async () => {
		const mockDotrain = 'mock dotrain content';
		(DotrainOrderGui.getOrderDetails as Mock).mockResolvedValueOnce({
			value: {
				name: 'Test Order',
				description: 'https://example.com/description.md',
				short_description: 'Test Short Description'
			}
		});
		// Mock fetch to reject
		mockFetch.mockRejectedValueOnce(new Error('Failed to fetch'));

		render(OrderPage, {
			props: {
				orderName: 'TestOrder',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Failed to fetch markdown')).toBeInTheDocument();
		});
	});

	it('renders markdown if description is a markdown url', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockOrderDetails = {
			value: {
				name: 'Test Order',
				description: 'https://example.com/description.md',
				short_description: 'Test Short Description'
			}
		};

		mockFetch.mockResolvedValueOnce({
			ok: true,
			text: () => Promise.resolve('mock markdown content')
		});

		(DotrainOrderGui.getOrderDetails as Mock).mockResolvedValueOnce(mockOrderDetails);

		render(OrderPage, {
			props: {
				orderName: 'TestOrder',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Order')).toBeInTheDocument();
			expect(screen.getByTestId('markdown-content')).toBeInTheDocument();
			expect(mockFetch).toHaveBeenCalledWith('https://example.com/description.md');
		});
	});

	it('falls back to plain text when markdown fetch fails', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockOrderDetails = {
			value: {
				name: 'Test Order',
				description: 'https://example.com/description.md',
				short_description: 'Test Short Description'
			}
		};

		mockFetch.mockResolvedValueOnce({
			ok: false,
			statusText: 'Not Found'
		});

		(DotrainOrderGui.getOrderDetails as Mock).mockResolvedValueOnce(mockOrderDetails);

		render(OrderPage, {
			props: {
				orderName: 'TestOrder',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Order')).toBeInTheDocument();
			expect(screen.getByTestId('plain-description')).toHaveTextContent(
				'https://example.com/description.md'
			);
		});
	});
});
