import { invoke } from '@tauri-apps/api';
import type { SgOrder, RainEvalResultsTable } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import { mockIPC } from '@tauri-apps/api/mocks';

export async function debugOrderQuote(
  order: SgOrder,
  inputIOIndex: number,
  outputIOIndex: number,
  orderbook: Hex,
  rpcUrl: string,
  blockNumber?: number,
) {
  return await invoke<[RainEvalResultsTable, string | undefined]>('debug_order_quote', {
    order,
    inputIoIndex: inputIOIndex,
    outputIoIndex: outputIOIndex,
    orderbook,
    rpcUrl,
    blockNumber,
  });
}

export const mockQuoteDebug: [RainEvalResultsTable, string | undefined] = [
  {
    columnNames: ['1', '2', '3'],
    rows: [['0x01', '0x02', '0x03']],
  },
  'some error msg',
];

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the trade_debug command correctly', async () => {
    mockIPC((cmd) => {
      if (cmd === 'debug_order_quote') {
        return mockQuoteDebug;
      }
    });

    const result = await debugOrderQuote(
      {
        id: '1',
        orderbook: { id: '0x00' },
        orderBytes: '0x123',
        orderHash: '0x123',
        owner: '0x123',
        outputs: [],
        inputs: [],
        active: true,
        addEvents: [],
        timestampAdded: '123',
        trades: [],
      } as unknown as SgOrder,
      0,
      0,
      '0x123',
      'https://rpc-url.com',
    );
    expect(result).toEqual(mockQuoteDebug);
  });
}
