import { expect, vi, describe, it } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import ModalConnect from './ModalConnect.svelte';
import type { NetworkCfg } from '@rainlanguage/orderbook';

const mockWalletConnectConnect = vi.hoisted(() =>
  vi.fn(async () => {
    return Promise.resolve();
  }),
);

vi.mock('$lib/stores/walletconnect', async () => {
  const { writable } = await import('svelte/store');
  return {
    walletconnectAccount: writable(undefined),
    walletconnectIsDisconnecting: writable(false),
    walletconnectIsConnecting: writable(false),
    walletconnectProvider: writable(undefined),
    walletConnectNetwork: writable(1),
    walletconnectConnect: mockWalletConnectConnect,
    walletconnectDisconnect: vi.fn(),
  };
});

vi.mock('$lib/stores/settings', async (importOriginal) => {
  return {
    ...((await importOriginal()) as object),
  };
});

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
  return {
    ...(await importOriginal()),
    useRaindexClient: vi.fn(() => ({
      getNetworkByChainId: vi.fn().mockReturnValue({ value: {} as NetworkCfg }),
      getAllNetworks: vi.fn().mockReturnValue({ value: new Map() }),
    })),
  };
});

describe('ModalConnect', () => {
  it('should reset after successful connection - walletconnect', async () => {
    render(ModalConnect);

    const button = screen.getByText('Connect to Wallet');
    fireEvent.click(button);

    const connectButton = await screen.findByText('Connect');
    expect(connectButton).toBeInTheDocument();
    fireEvent.click(connectButton);

    expect(mockWalletConnectConnect).toHaveBeenCalled();

    expect(await screen.findByText('Connect to Wallet')).toBeInTheDocument();
  });

});
