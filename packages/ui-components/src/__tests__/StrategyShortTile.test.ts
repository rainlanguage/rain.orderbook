import { vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, beforeEach } from 'vitest';
import StrategyShortTile from '../lib/components/deployment/StrategyShortTile.svelte';
import { mockPageStore } from '$lib/__mocks__/stores';

// Mock using the hoisted store
vi.mock('$app/stores', async () => {
	const { mockPageStore } = await import('$lib/__mocks__/stores');
	return {
		page: mockPageStore
	};
});

describe('StrategyShortTile', () => {
	const mockStrategyDetails = {
		name: 'Test Strategy',
		description: 'A test strategy full description',
		short_description: 'A test strategy description'
	};

	beforeEach(() => {
		// Reset page URL params before each test
		mockPageStore.mockSetSubscribeValue({
			url: new URL('http://localhost:3000'),
			params: {},
			route: { id: '' },
			status: 200,
			error: null,
			data: {},
			form: undefined,
			state: {
				page: 1,
				perPage: 10,
				total: 100
			}
		});
	});

	it('renders strategy name and description', () => {
		render(StrategyShortTile, {
			props: {
				strategyDetails: mockStrategyDetails,
				registryName: 'test-registry'
			}
		});

		expect(screen.getByText('Test Strategy')).toBeInTheDocument();
		expect(screen.getByText('A test strategy description')).toBeInTheDocument();
	});

	it('generates correct href without registry parameter', () => {
		render(StrategyShortTile, {
			props: {
				strategyDetails: mockStrategyDetails,
				registryName: 'test-registry'
			}
		});

		const link = screen.getByRole('link');
		expect(link.getAttribute('href')).toBe('/deploy/test-registry');
	});

	it('generates correct href with registry parameter', () => {
		mockPageStore.mockSetSubscribeValue({
			url: new URL('http://localhost:3000?registry=custom-registry'),
			params: {},
			route: { id: '' },
			status: 200,
			error: null,
			data: {},
			form: undefined,
			state: {
				page: 1,
				perPage: 10,
				total: 100
			}
		});

		render(StrategyShortTile, {
			props: {
				strategyDetails: mockStrategyDetails,
				registryName: 'test-registry'
			}
		});

		const link = screen.getByRole('link');
		expect(link.getAttribute('href')).toBe('/deploy/test-registry?registry=custom-registry');
	});

	it('applies correct styling classes', () => {
		render(StrategyShortTile, {
			props: {
				strategyDetails: mockStrategyDetails,
				registryName: 'test-registry'
			}
		});

		const link = screen.getByRole('link');
		expect(link).toHaveClass(
			'flex',
			'flex-col',
			'gap-y-2',
			'rounded-xl',
			'border',
			'border-gray-200',
			'p-4',
			'hover:bg-gray-50',
			'dark:border-gray-800',
			'dark:hover:bg-gray-900'
		);
	});
});
