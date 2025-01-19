import { expect, vi, describe, it } from 'vitest';
import ModalExecute from './ModalExecute.svelte';
import { render, screen } from '@testing-library/svelte';
import { settings } from '$lib/stores/settings';

vi.mock('$lib/stores/walletconnect', async () => {
  const { writable } = await import('svelte/store');
  return {
    walletconnectAccount: writable('0x123'),
    walletconnectIsDisconnecting: writable(false),
    walletconnectIsConnecting: writable(false),
    walletconnectProvider: writable(undefined),
    walletConnectNetwork: writable(1),
    walletConnectConnect: vi.fn(),
    walletconnectDisconnect: vi.fn(),
  };
});

vi.mock('$lib/stores/settings', async (importOriginal) => {
  return {
    ...((await importOriginal()) as object),
  };
});

describe('ModalExecute', () => {
  describe('should show network connection error if wallet is connected to wrong network', () => {
    it('should show unknown network name', () => {
      render(ModalExecute, {
        props: {
          open: true,
          title: 'Test',
          execButtonLabel: 'Execute',
          executeLedger: vi.fn(),
          executeWalletconnect: vi.fn(),
          isSubmitting: false,
          overrideNetwork: {
            key: 'test',
            rpc: 'https://test.com',
            'chain-id': 2,
          },
        },
      });

      const errorElement = screen.getByTestId('network-connection-error');
      expect(errorElement).toHaveTextContent(
        'You are connected to an unknown network. Please connect your wallet to test network.',
      );
    });

    it('should show current connected network name', () => {
      settings.set({
        networks: {
          mainnet: {
            'chain-id': 1,
            rpc: 'https://mainnet.com',
          },
        },
      });

      render(ModalExecute, {
        props: {
          open: true,
          title: 'Test',
          execButtonLabel: 'Execute',
          executeLedger: vi.fn(),
          executeWalletconnect: vi.fn(),
          isSubmitting: false,
          overrideNetwork: {
            key: 'test',
            rpc: 'https://test.com',
            'chain-id': 2,
          },
        },
      });

      const errorElement = screen.getByTestId('network-connection-error');
      expect(errorElement).toHaveTextContent(
        'You are connected to mainnet network. Please connect your wallet to test network.',
      );
    });
  });
});
