import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import InvalidStrategiesSection from '../lib/components/deployment/InvalidStrategiesSection.svelte';
import type { StrategyDetail } from '$lib/types/strategy';

describe('InvalidStrategiesSection', () => {
	const mockInvalidStrategies: StrategyDetail[] = [
		{
			name: 'Strategy 1',
			error: 'Invalid configuration',
			details: {
				name: 'Strategy 1',
				description: 'Description 1',
				short_description: 'Short description 1'
			},
			dotrain: ''
		},
		{
			name: 'Strategy 2',
			error: 'Missing required field',
			details: {
				name: 'Strategy 2',
				description: 'Description 2',
				short_description: 'Short description 2'
			},
			dotrain: ''
		}
	];

	it('renders the section title', () => {
		render(InvalidStrategiesSection, { props: { invalidStrategies: [] } });
		expect(screen.getByText('Invalid Strategies in registry')).toBeInTheDocument();
	});

	it('displays no strategies when array is empty', () => {
		render(InvalidStrategiesSection, { props: { invalidStrategies: [] } });
		const container = screen.getByTestId('invalid-strategies');
		expect(container.querySelectorAll('.flex.flex-col.gap-1')).toHaveLength(0);
	});

	it('displays multiple invalid strategies with their errors', () => {
		render(InvalidStrategiesSection, { props: { invalidStrategies: mockInvalidStrategies } });

		expect(screen.getByText('Strategy 1')).toBeInTheDocument();
		expect(screen.getByText('Strategy 2')).toBeInTheDocument();

		expect(screen.getByText('Invalid configuration')).toBeInTheDocument();
		expect(screen.getByText('Missing required field')).toBeInTheDocument();
	});
});
