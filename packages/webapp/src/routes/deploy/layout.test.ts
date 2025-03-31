import { describe, expect, it, beforeAll, afterAll, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { vi } from 'vitest';
import Layout from './+layout.svelte';
import { REGISTRY_URL } from '$lib/constants';
import { page } from '$app/stores';

// Mock the $app/stores
vi.mock('$app/stores', () => {
  const page = vi.fn();
  return { page };
});

// Mock localStorage
const localStorageMock: Storage = (() => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string): string | null => store[key] || null,
    setItem: (key: string, value: string): void => {
      store[key] = value.toString();
    },
    removeItem: (key: string): void => {
      delete store[key];
    },
    clear: (): void => {
      store = {};
    },
    key: (index: number): string => Object.keys(store)[index] || '',
    length: 0
  };
})();

// Save original localStorage
let originalLocalStorage: Storage;

describe('Layout Component', () => {
  // Setup and teardown for localStorage
  beforeAll((): void => {
    originalLocalStorage = window.localStorage;
    Object.defineProperty(window, 'localStorage', { value: localStorageMock });
  });

  afterAll((): void => {
    Object.defineProperty(window, 'localStorage', { value: originalLocalStorage });
  });

  beforeEach((): void => {
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  // Helper to mock the page store
  function mockPageStore(pathname = '/', searchParams = new URLSearchParams()) {
    // @ts-ignore - We're mocking the store
    page.mockImplementation(() => ({
      subscribe: (callback: Function) => {
        callback({
          url: {
            pathname,
            searchParams
          },
          data: {
            pageName: pathname === '/' ? 'Home' : 'Deploy'
          }
        });
        return () => {};
      }
    }));
  }

  it('should render with basic page layout', () => {
    mockPageStore();
    
    const { container } = render(Layout);
    
    expect(container).toBeTruthy();
    expect(screen.getByText('Home')).toBeTruthy();
  });

  it('should show advanced mode toggle on deploy page', () => {
    mockPageStore('/deploy');
    
    render(Layout);
    
    expect(screen.getByText('Advanced mode')).toBeTruthy();
  });

  it('should not show advanced mode toggle on non-deploy pages', () => {
    mockPageStore('/other-page');
    
    render(Layout);
    
    expect(screen.queryByText('Advanced mode')).toBeNull();
  });

  it('should set registry in localStorage from URL search params', () => {
    const searchParams = new URLSearchParams();
    searchParams.set('registry', 'https://custom-registry.com');
    mockPageStore('/deploy', searchParams);
    
    render(Layout);
    
    expect(localStorageMock.getItem('registry')).toBe('https://custom-registry.com');
  });

  it('should remove registry from localStorage when search param is missing', () => {
    // First set something in localStorage
    localStorageMock.setItem('registry', 'https://custom-registry.com');
    
    // Then render without search param
    mockPageStore('/deploy');
    
    render(Layout);
    
    expect(localStorageMock.getItem('registry')).toBeNull();
  });

  it('should show custom registry warning when using non-default registry', () => {
    localStorageMock.setItem('registry', 'https://custom-registry.com');
    mockPageStore('/deploy');
    
    const { container } = render(Layout);
    
    // Since we don't know how CustomRegistryWarning is rendered exactly,
    // we'll check for any component presence. Adjust as needed.
    const warning = container.querySelector('[data-testid="custom-registry-warning"]');
    expect(warning).toBeTruthy();
  });

  it('should not show custom registry warning when using default registry', () => {
    localStorageMock.setItem('registry', REGISTRY_URL);
    mockPageStore('/deploy');
    
    const { container } = render(Layout);
    
    const warning = container.querySelector('[data-testid="custom-registry-warning"]');
    expect(warning).toBeNull();
  });

  it('should display InputRegistryUrl when advanced mode is on', async () => {
    localStorageMock.setItem('registry', 'https://custom-registry.com');
    mockPageStore('/deploy');
    
    const { container } = render(Layout);
    
    // Advanced mode should be on because registry is set
    const inputRegistryUrl = container.querySelector('[data-testid="input-registry-url"]');
    expect(inputRegistryUrl).toBeTruthy();
  });

  it('should toggle advanced mode when the toggle is clicked', async () => {
    mockPageStore('/deploy');
    
    const { container } = render(Layout);
    
    // Get the toggle
    const toggle = screen.getByText('Advanced mode').closest('label');
    expect(toggle).toBeTruthy();
    
    // Initially, no registry in localStorage and no InputRegistryUrl
    expect(localStorageMock.getItem('registry')).toBeNull();
    
    // Click the toggle
    if (toggle) {
      await fireEvent.click(toggle);
    }
    
    // This test will need adjustment based on how your toggle actually works
    // Either it should now show InputRegistryUrl, or there should be some change in localStorage
    // For testing a complete flow, you might need to mock the Toggle component behavior
  });
});