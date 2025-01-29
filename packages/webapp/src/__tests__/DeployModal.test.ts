import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import DeployModal from '../lib/components/DeployModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import type { ComponentProps } from 'svelte';

export type DeployModalProps = ComponentProps<DeployModal>;
// Mock the stores
vi.mock('@rainlanguage/ui-components', () => ({
  transactionStore: {
    handleDeploymentTransaction: vi.fn()
  }
}));

vi.mock('$lib/stores/wagmi', () => ({
  wagmiConfig: {
    subscribe: vi.fn()
  }
}));

describe('DeployModal', () => {
  const mockProps = {
    open: true,
    approvals: {
      approvalCalldata: '0x',
      token: '0x',
      spender: '0x'
    },
    deploymentCalldata: {
      calldata: '0x',
      value: 0n
    },
    orderbookAddress: '0x123' as const,
    chainId: 1
  } as unknown as DeployModalProps

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders TransactionModal with correct messages', () => {
    render(DeployModal, { props: mockProps });

    expect(screen.getByText('Deploying your strategy...')).toBeTruthy();
  });

  it('calls handleDeploymentTransaction with correct parameters', () => {
    render(DeployModal, { props: mockProps });

    expect(transactionStore.handleDeploymentTransaction).toHaveBeenCalledWith({
      config: expect.any(Object),
      approvals: mockProps.approvals,
      deploymentCalldata: mockProps.deploymentCalldata,
      orderbookAddress: mockProps.orderbookAddress,
      chainId: mockProps.chainId
    });
  });

  it('updates open prop correctly', async () => {
    const { component } = render(DeployModal, { props: mockProps });

    // Test two-way binding of open prop
    await component.$set({ open: false });
    expect(component.open).toBe(false);
  });
});