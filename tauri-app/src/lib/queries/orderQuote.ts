import { invoke } from '@tauri-apps/api';
import type { RainEvalResultsTable, RaindexOrder } from '@rainlanguage/orderbook';
import { mockIPC } from '@tauri-apps/api/mocks';

export async function debugOrderQuote(
  order: RaindexOrder,
  inputIOIndex: number,
  outputIOIndex: number,
  blockNumber?: number,
) {
  return await invoke<[RainEvalResultsTable, string | undefined]>('debug_order_quote', {
    order,
    inputIoIndex: inputIOIndex,
    outputIoIndex: outputIOIndex,
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
        timestampAdded: BigInt(123),
        tradesCount: 0,
      } as unknown as RaindexOrder,
      0,
      0,
      123,
    );
    expect(result).toEqual(mockQuoteDebug);
  });
}
