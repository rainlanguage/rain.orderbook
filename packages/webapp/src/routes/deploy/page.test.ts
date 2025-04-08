import { render, screen } from '@testing-library/svelte';
import { vi, describe, it, expect } from 'vitest';
import DeployPage from './+page.svelte';
import type { ComponentProps } from 'svelte';

type PageProps = ComponentProps<DeployPage>;

vi.mock('@rainlanguage/ui-components', async () => {
	const MockComponent = (await import('../../lib/__mocks__/MockComponent.svelte')).default;

	return {
		ValidStrategiesSection: MockComponent,
		InvalidStrategiesSection: MockComponent
	};
});

const defaultProps: PageProps = {
	data: {
		error: null,
		validStrategies: [],
		invalidStrategies: []
	}
} as unknown as PageProps;

describe('Deploy Page', () => {
	it('should display error message when error is present', async () => {
		render(DeployPage, {
			props: {
				data: {
					error: 'Test error message',
					validStrategies: [],
					invalidStrategies: []
				}
			} as PageProps
		});
		expect(screen.getByText('Error loading registry:')).toBeInTheDocument();
		expect(screen.getByText('Test error message')).toBeInTheDocument();
	});

	it('should display no strategies message when both arrays are empty', async () => {
		render(DeployPage, { props: defaultProps });

		expect(screen.getByText('No strategies found')).toBeInTheDocument();
	});

	it('should render ValidStrategiesSection when valid strategies exist', async () => {
		render(DeployPage, {
			props: {
				data: {
					...defaultProps.data,
					validStrategies: [{ name: 'Strategy 1' }]
				} as unknown as PageProps['data']
			}
		});

		expect(screen.queryByText('No strategies found')).not.toBeInTheDocument();
		expect(screen.getByTestId('valid-strategies-section')).toBeInTheDocument();
	});

	it('should render InvalidStrategiesSection when invalid strategies exist', async () => {
		render(DeployPage, {
			props: {
				data: {
					...defaultProps.data,
					invalidStrategies: [{ name: 'Invalid Strategy', error: 'Some error' }]
				} as unknown as PageProps['data']
			}
		});

		expect(screen.queryByText('No strategies found')).not.toBeInTheDocument();
		expect(screen.getByTestId('invalid-strategies-section')).toBeInTheDocument();
	});

	it('should display the intro text about Raindex', () => {
		render(DeployPage, { props: defaultProps });

		expect(
			screen.getByText(/Raindex empowers you to take full control of your trading strategies/)
		).toBeInTheDocument();
	});
});
