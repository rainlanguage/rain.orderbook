import { invoke } from '@tauri-apps/api';
import { mockIPC } from '@tauri-apps/api/mocks';

export const tradeDebug = async (txHash: string, rpcUrl: string) => {
  return await invoke('debug_trade', {
    txHash,
    rpcUrl,
  });
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the trade_debug command correctly', async () => {
    const mockTradeDebug = [1, 2, 3];

    mockIPC((cmd) => {
      if (cmd === 'trade_debug') {
        return mockTradeDebug;
      }
    });

    const result = await tradeDebug('0x123', 'https://rpc-url.com');
    expect(result).toEqual(mockTradeDebug);
  });
}
