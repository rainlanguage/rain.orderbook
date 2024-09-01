import { invoke } from '@tauri-apps/api';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { Hex } from 'viem';

export const tradeDebug = async (txHash: string, rpcUrl: string) => {
  return await invoke<Hex[]>('debug_trade', {
    txHash,
    rpcUrl,
  });
};

export const mockTradeDebug = ['0x01', '0x02', '0x03'];

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the trade_debug command correctly', async () => {
    mockIPC((cmd) => {
      if (cmd === 'debug_trade') {
        return mockTradeDebug;
      }
    });

    const result = await tradeDebug('0x123', 'https://rpc-url.com');
    expect(result).toEqual(mockTradeDebug);
  });
}
