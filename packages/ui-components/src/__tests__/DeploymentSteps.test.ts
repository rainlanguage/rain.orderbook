import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import DeploymentSteps from '../lib/components/deployment/DeploymentSteps.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import testStrategy from '../lib/components/deployment/test-strategy.rain?raw';

vi.mock('@rainlanguage/orderbook/js_api', () => ({
  DotrainOrderGui: {
    getAvailableDeployments: vi.fn(),
    chooseDeployment: vi.fn()
  }
}));

describe('DeploymentSteps', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders load strategy button', () => {
    render(DeploymentSteps);
    expect(screen.getByText('Load Strategy')).toBeInTheDocument();
  });

  it('loads strategy when button is clicked', async () => {
    render(DeploymentSteps);
    const loadButton = screen.getByText('Load Strategy');


    const mockDeployments = [
      { key: 'deployment1', label: 'Deployment 1' },
      { key: 'deployment2', label: 'Deployment 2' }
    ];

    (DotrainOrderGui.getAvailableDeployments as Mock).mockResolvedValue(mockDeployments);

    await fireEvent.click(loadButton);


    expect(DotrainOrderGui.getAvailableDeployments).toHaveBeenCalled();
  });

  it('shows deployments dropdown after strategy is loaded', async () => {
    render(DeploymentSteps);
    const loadButton = screen.getByText('Load Strategy');


    const mockDeployments = [
      { key: 'deployment1', label: 'Deployment 1' },
      { key: 'deployment2', label: 'Deployment 2' }
    ];

    (DotrainOrderGui.getAvailableDeployments as Mock).mockResolvedValue(mockDeployments);

    await fireEvent.click(loadButton);

    expect(screen.getByText('Deployments')).toBeInTheDocument();
    expect(screen.getByText('Select a deployment')).toBeInTheDocument();
  });

  it('initializes GUI when deployment is selected', async () => {
    render(DeploymentSteps);

    const mockDeployments = [
      { key: 'deployment1', label: 'Deployment 1' },
      { key: 'deployment2', label: 'Deployment 2' }
    ];


    const mockGui = {
      getTokenInfos: vi.fn().mockReturnValue({}),
      getCurrentDeployment: vi.fn().mockReturnValue({
        deposits: [],
        deployment: {
          order: {
            inputs: [],
            outputs: []
          }
        }
      }),
      getAllFieldDefinitions: vi.fn().mockReturnValue([])
    };
    const loadButton = screen.getByText('Load Strategy');

    (DotrainOrderGui.getAvailableDeployments as Mock).mockResolvedValue(mockDeployments);

    await fireEvent.click(loadButton);

    expect(screen.getByText('Deployments')).toBeInTheDocument();
    expect(screen.getByText('Select a deployment')).toBeInTheDocument();

    (DotrainOrderGui.chooseDeployment as Mock).mockResolvedValue(mockGui);

    const dropdownButton = screen.getByTestId('dropdown-button');
    await fireEvent.click(dropdownButton);
    const dropdown = screen.getByTestId('dropdown');
    await fireEvent.click(dropdown);
    const deploymentOption = screen.getByText('deployment1');
    await fireEvent.click(deploymentOption);

    expect(DotrainOrderGui.chooseDeployment).toHaveBeenCalled();
  });

  it('handles errors during deployment initialization', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    (DotrainOrderGui.getAvailableDeployments as Mock).mockRejectedValue(new Error('Failed to load'));

    render(DeploymentSteps);
    const loadButton = screen.getByText('Load Strategy');
    await fireEvent.click(loadButton);

    expect(consoleSpy).toHaveBeenCalledWith('Failed to load deployments:', expect.any(Error));
    consoleSpy.mockRestore();
  });
});