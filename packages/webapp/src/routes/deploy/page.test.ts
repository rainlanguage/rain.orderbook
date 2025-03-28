import { render, cleanup } from '@testing-library/svelte';
import { vi, describe, it, expect, afterEach } from 'vitest';
import Page from './+page.svelte';

import type { ComponentProps } from 'svelte';

type PageProps = ComponentProps<Page>

vi.mock('@rainlanguage/ui-components', async () => {
	const MockComponent = (await import('../../lib/__mocks__/MockComponent.svelte')).default;
  return {
    ValidStrategiesSection: MockComponent,
    InvalidStrategiesSection: MockComponent
  };
});

describe('Strategies Page', () => {
  afterEach(() => {
    cleanup();
  });

  it.only('displays error message when error is present', () => {
    const errorMessage = 'Failed to connect to registry';
    const { getByText } = render(Page, {
      data: {
        error: errorMessage,
        validStrategies: [],
        invalidStrategies: []
      }
    } as PageProps);

    expect(getByText('Error loading registry:')).toBeInTheDocument();
    expect(getByText(errorMessage)).toBeInTheDocument();
  });

	it('displays info section when no error is present', () => {
		const { getByText } = render(Page, {
			data: {
				error: null,
				validStrategies: [],
				invalidStrategies: []
			}
		} as unknown as PageProps);


    expect(getByText(/Raindex empowers you to take full control/)).toBeInTheDocument();
    expect(getByText('Rainlang')).toBeInTheDocument();
  });

  it('displays "No strategies found" when both valid and invalid strategies are empty', () => {
    const { getByText } = render(Page, {
      data: {
        error: null,
        validStrategies: [],
        invalidStrategies: []
      }
    } as unknown as PageProps);

    expect(getByText('No strategies found')).toBeInTheDocument();
  });

  it('renders ValidStrategiesSection when valid strategies exist', () => {
    const validStrategies = [{ id: '1', name: 'Strategy 1' }];
    const { container } = render(Page, {
      data: {
        error: null,
        validStrategies,
        invalidStrategies: []
      }
    } as unknown as PageProps);

    expect(container.querySelector('[data-testid="mock-component"]')).toBeInTheDocument();
  });

  it('renders InvalidStrategiesSection when invalid strategies exist', () => {
    const invalidStrategies = [{ id: '1', name: 'Strategy 1', error: 'Invalid strategy' }];
    const { container } = render(Page, {
      data: {
        error: null,
        validStrategies: [],
        invalidStrategies
      }
    } as unknown as PageProps);

    expect(container.querySelector('[data-testid="mock-component"]')).toBeInTheDocument();
  });

  it('renders both sections when both valid and invalid strategies exist', () => {
    const validStrategies = [{ id: '1', name: 'Valid Strategy' }];
    const invalidStrategies = [{ id: '2', name: 'Invalid Strategy', error: 'Error' }];
    const { container } = render(Page, {
      data: {
        error: null,
        validStrategies,
        invalidStrategies
      }
    } as unknown as PageProps);

    const mockComponents = container.querySelectorAll('[data-testid="mock-component"]');
    expect(mockComponents.length).toBeGreaterThanOrEqual(2);
  });
});