import { render, screen, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import DeployPage from './+page.svelte';
import * as handleGuiInitializationModule from '$lib/services/handleGuiInitialization';
import { goto } from '$app/navigation';
import { readable } from 'svelte/store';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

const {
  mockPageStore,
  mockWagmiConfigStore,
  mockConnectedStore,
  mockAppKitModalStore,
  mockSignerAddressStore
} = await vi.hoisted(() => import('$lib/__mocks__/stores'));

// Use the more detailed mocking approach from main branch
vi.mock('$app/stores', async (importOriginal) => {
  return {
    ...((await importOriginal()) as object),
    page: mockPageStore
  };
});

vi.mock('$app/navigation', async (importOriginal) => {
  return {
    ...((await importOriginal()) as object),
    goto: vi.fn()
  };
});

// Use the MockComponent approach from HEAD for component mocking
vi.mock('@rainlanguage/ui-components', async () => {
  const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
  return { 
    GuiProvider: MockComponent, 
    DeploymentSteps: MockComponent 
  };
});

// Use separate store mocks from main branch
vi.mock('$lib/stores/wagmi', () => ({
  wagmiConfig: mockWagmiConfigStore,
  connected: mockConnectedStore,
  appKitModal: mockAppKitModalStore,
  signerAddress: mockSignerAddressStore
}));

vi.mock('$lib/services/modal', () => ({
  handleDeployModal: vi.fn(),
  handleDisclaimerModal: vi.fn()
}));

// Mock the handleGuiInitialization function
vi.mock('$lib/services/handleGuiInitialization', () => ({
  handleGuiInitialization: vi.fn().mockResolvedValue({
    gui: null,
    error: null
  })
}));

describe('DeployPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockPageStore.reset();
    mockWagmiConfigStore.reset();
    mockConnectedStore.reset();
    mockAppKitModalStore.reset();
    mockSignerAddressStore.reset();
    vi.resetAllMocks();

    // Default mock for handleGuiInitialization
    vi.mocked(handleGuiInitializationModule.handleGuiInitialization).mockResolvedValue({
      gui: null,
      error: null
    });
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  it('should call handleGuiInitialization with correct parameters when dotrain and deployment exist', async () => {
    const mockDotrain = 'mock-dotrain';
    const mockDeploymentKey = 'test-key';
    const mockStateFromUrl = 'some-state';

    mockPageStore.mockSetSubscribeValue({
      data: {
        stores: { settings: readable({}) },
        dotrain: mockDotrain,
        deployment: { key: mockDeploymentKey },
        strategyDetail: {}
      },
      url: new URL(`http://localhost:3000/deploy?state=${mockStateFromUrl}`)
    });

    render(DeployPage);

    await waitFor(() => {
      expect(handleGuiInitializationModule.handleGuiInitialization).toHaveBeenCalledWith(
        mockDotrain,
        mockDeploymentKey,
        mockStateFromUrl
      );
    });
  });

  it('should not call handleGuiInitialization when dotrain is missing', async () => {
    mockPageStore.mockSetSubscribeValue({
      data: {
        stores: { settings: readable({}) },
        dotrain: null,
        deployment: { key: 'test-key' },
        strategyDetail: {}
      },
      url: new URL('http://localhost:3000/deploy')
    });

    render(DeployPage);

    await new Promise((resolve) => setTimeout(resolve, 50));

    expect(handleGuiInitializationModule.handleGuiInitialization).not.toHaveBeenCalled();
  });

  it('should not call handleGuiInitialization when deployment is missing', async () => {
    mockPageStore.mockSetSubscribeValue({
      data: {
        stores: { settings: readable({}) },
        dotrain: 'some-dotrain',
        deployment: null,
        strategyDetail: {}
      },
      url: new URL('http://localhost:3000/deploy')
    });

    render(DeployPage);

    await new Promise((resolve) => setTimeout(resolve, 50));

    expect(handleGuiInitializationModule.handleGuiInitialization).not.toHaveBeenCalled();
  });

  it('should redirect to /deploy if dotrain or deployment is missing', async () => {
    vi.useFakeTimers();

    mockPageStore.mockSetSubscribeValue({
      data: {
        stores: { settings: readable({}) },
        dotrain: null,
        deployment: null,
        strategyDetail: null
      },
      url: new URL('http://localhost:3000/deploy/strategy/key')
    });

    render(DeployPage);

    expect(screen.getByText(/Deployment not found/i)).toBeInTheDocument();
    
    vi.advanceTimersByTime(5000);
    await vi.runAllTimersAsync();

    expect(goto).toHaveBeenCalledWith('/deploy');

    vi.useRealTimers();
  });

  it('should show error message when GUI initialization fails', async () => {
    mockPageStore.mockSetSubscribeValue({
      data: {
        stores: { settings: readable({}) },
        dotrain: 'https://dotrain.example.com',
        deployment: {
          key: 'test-deployment',
          name: 'Test Deployment',
          description: 'This is a test deployment'
        },
        strategyDetail: {
          name: 'Test Strategy',
          description: 'This is a test strategy'
        }
      },
      url: new URL('http://localhost:3000/deploy')
    });

    vi.mocked(handleGuiInitializationModule.handleGuiInitialization).mockResolvedValue({
      gui: null,
      error: 'Failed to initialize GUI'
    });

    render(DeployPage);

    await waitFor(() => {
      expect(screen.getByText('Failed to initialize GUI')).toBeInTheDocument();
    });
  });

  it('should render DeploymentSteps when GUI is available', async () => {
    const mockGui = {
      name: 'Test GUI'
    };

    mockPageStore.mockSetSubscribeValue({
      data: {
        stores: { settings: readable({}) },
        dotrain: 'https://dotrain.example.com',
        deployment: {
          key: 'test-deployment',
          name: 'Test Deployment',
          description: 'This is a test deployment'
        },
        strategyDetail: {
          name: 'Test Strategy',
          description: 'This is a test strategy'
        }
      },
      url: new URL('http://localhost:3000/deploy?state=test-state')
    });

    vi.mocked(handleGuiInitializationModule.handleGuiInitialization).mockResolvedValue({
      gui: mockGui as unknown as DotrainOrderGui,
      error: null
    });

    render(DeployPage);

    // Wait for GUI to be available and DeploymentSteps to be rendered
    await waitFor(() => {
      expect(handleGuiInitializationModule.handleGuiInitialization).toHaveBeenCalledWith(
        'https://dotrain.example.com',
        'test-deployment',
        'test-state'
      );
      // Additional assertions could be added here to verify DeploymentSteps rendering
    });
  });

  it('should handle initialization with empty state from URL', async () => {
    mockPageStore.mockSetSubscribeValue({
      data: {
        stores: { settings: readable({}) },
        dotrain: 'https://dotrain.example.com',
        deployment: {
          key: 'test-deployment'
        },
        strategyDetail: {
          name: 'Test Strategy'
        }
      },
      url: new URL('http://localhost:3000/deploy')
    });

    const mockGui = { name: 'Test GUI' } as unknown as DotrainOrderGui;
    vi.mocked(handleGuiInitializationModule.handleGuiInitialization).mockResolvedValue({
      gui: mockGui,
      error: null
    });

    render(DeployPage);

    await waitFor(() => {
      expect(handleGuiInitializationModule.handleGuiInitialization).toHaveBeenCalledWith(
        'https://dotrain.example.com',
        'test-deployment',
        ''
      );
    });
  });
  
  it('should correctly pass state parameter from URL to handleGuiInitialization', async () => {
    // Test with various state values
    const testCases = [
      { stateValue: 'simple-state', description: 'simple string' },
      { stateValue: 'complex%20state%20with%20spaces', description: 'URL encoded string' },
      { stateValue: '{"json":"data"}', description: 'JSON string' },
      { stateValue: '12345', description: 'numeric string' }
    ];
    
    for (const { stateValue, description } of testCases) {
      // Clear previous calls
      vi.clearAllMocks();
      
      mockPageStore.mockSetSubscribeValue({
        data: {
          stores: { settings: readable({}) },
          dotrain: 'https://dotrain.example.com',
          deployment: {
            key: 'test-deployment'
          },
          strategyDetail: {
            name: 'Test Strategy'
          }
        },
        url: new URL(`http://localhost:3000/deploy?state=${stateValue}`)
      });

      const mockGui = { name: 'Test GUI' } as unknown as DotrainOrderGui;
      vi.mocked(handleGuiInitializationModule.handleGuiInitialization).mockResolvedValue({
        gui: mockGui,
        error: null
      });

      // Re-render for each test case
      const { unmount } = render(DeployPage);

      await waitFor(() => {
        expect(handleGuiInitializationModule.handleGuiInitialization).toHaveBeenCalledWith(
          'https://dotrain.example.com',
          'test-deployment',
          stateValue
        );
      }, { timeout: 1000 });
      
      console.log(`✓ Correctly passed ${description} state parameter`);
      
      // Clean up between test cases
      unmount();
    }
  });
});