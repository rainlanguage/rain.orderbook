import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, type Mock } from 'vitest';
import CustomRegistryWarning from '$lib/components/CustomRegistryWarning.svelte';
import { writable } from 'svelte/store';
import { useRegistry } from '@rainlanguage/ui-components';


const mockRegistry = vi.fn()
const mockResetToDefault = vi.fn()

vi.mock('@rainlanguage/ui-components', () => {
  return {
    useRegistry: vi.fn()
  };
});

describe('CustomRegistryWarning Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
	(useRegistry as Mock).mockReturnValue(mockRegistry)
	mockRegistry.mockReturnValue(writable({
		resetToDefault: mockResetToDefault
	}))
	
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

  it('should call resetToDefault when "Use default" is clicked', async () => {
    const mockStore = vi.mocked(useRegistry)();
    let mockRegistryManager;
    

    const unsubscribe = mockStore.subscribe(value => {
      mockRegistryManager = value;
    });
    unsubscribe();

    render(CustomRegistryWarning);

    const defaultLink = screen.getByText('Use default.');
    await fireEvent.click(defaultLink);


    expect(mockRegistryManager).toHaveBeenCalledTimes(1);
  });
});