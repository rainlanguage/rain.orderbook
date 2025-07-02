import type { RainEvalResultsTable } from '@rainlanguage/orderbook';
import { invoke } from '@tauri-apps/api';
import { mockIPC } from '@tauri-apps/api/mocks';

export const tradeDebug = async (txHash: string, rpcUrls: string[]) => {
  return await invoke<RainEvalResultsTable>('debug_trade', {
    txHash,
    rpcs: rpcUrls,
  });
};

export const mockTradeDebug: RainEvalResultsTable = {
  columnNames: ['1', '2', '3'],
  rows: [['0x01', '0x02', '0x03']],
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the trade_debug command correctly', async () => {
    mockIPC((cmd) => {
      if (cmd === 'debug_trade') {
        return mockTradeDebug;
      }
    });

    const result = await tradeDebug('0x123', ['https://rpc-url.com']);
    expect(result).toEqual(mockTradeDebug);
  });
}
