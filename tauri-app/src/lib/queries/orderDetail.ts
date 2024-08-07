import type { OrderDetailExtended } from '$lib/typeshare/orderDetail';
import { invoke } from '@tauri-apps/api';
import { mockIPC } from '@tauri-apps/api/mocks';

export type OrderDetailArgs = {
  id: string;
  subgraphArgs: {
    url: string;
  };
};

export const orderDetail = async (id: string, url: string | undefined) => {
  if (!url) {
    return undefined;
  }
  return await invoke<OrderDetailExtended>('order_detail', {
    id,
    subgraphArgs: { url },
  } as OrderDetailArgs);
};

export const mockOrderDetailsExtended: OrderDetailExtended[] = [
  {
    order: {
      id: 'order1',
      order_bytes: '0x123456',
      order_hash: '0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef',
      owner: '0x1111111111111111111111111111111111111111',
      outputs: [
        {
          id: 'vault1',
          token: {
            id: 'token1',
            address: '0xaaaaaa1111111111111111111111111111111111',
            name: 'Token1',
            symbol: 'TK1',
            decimals: '18',
          },
          balance: '1000',
          vault_id: '0x1111111111111111111111111111111111111111111111111111111111111111',
        },
      ],
      inputs: [
        {
          id: 'vault2',
          token: {
            id: 'token2',
            address: '0xbbbbbb2222222222222222222222222222222222',
            name: 'Token2',
            symbol: 'TK2',
            decimals: '18',
          },
          balance: '500',
          vault_id: '0x2222222222222222222222222222222222222222222222222222222222222222',
        },
      ],
      active: true,
      add_events: [
        {
          transaction: {
            block_number: '12345',
            timestamp: '1620000000',
          },
        },
      ],
      meta: 'metadata1',
      timestamp_added: '1620000000',
    },
    rainlang: 'rainlang1',
  },
  {
    order: {
      id: 'order2',
      order_bytes: '0x654321',
      order_hash: '0xbbbbbbb1bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
      owner: '0x2222222222222222222222222222222222222222',
      outputs: [
        {
          id: 'vault3',
          token: {
            id: 'token3',
            address: '0xcccccc3333333333333333333333333333333333',
            name: 'Token3',
            symbol: 'TK3',
            decimals: '18',
          },
          balance: '2000',
          vault_id: '0x3333333333333333333333333333333333333333333333333333333333333333',
        },
      ],
      inputs: [
        {
          id: 'vault4',
          token: {
            id: 'token4',
            address: '0xdddddd4444444444444444444444444444444444',
            name: 'Token4',
            symbol: 'TK4',
            decimals: '18',
          },
          balance: '1500',
          vault_id: '0x4444444444444444444444444444444444444444444444444444444444444444',
        },
      ],
      active: false,
      add_events: [
        {
          transaction: {
            block_number: '12346',
            timestamp: '1620000010',
          },
        },
      ],
      meta: 'metadata2',
      timestamp_added: '1620000010',
    },
    rainlang: 'rainlang2',
  },
  {
    order: {
      id: 'order3',
      order_bytes: '0xabcdef',
      order_hash: '0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',
      owner: '0x3333333333333333333333333333333333333333',
      outputs: [
        {
          id: 'vault5',
          token: {
            id: 'token5',
            address: '0xeeeeee5555555555555555555555555555555555',
            name: 'Token5',
            symbol: 'TK5',
            decimals: '18',
          },
          balance: '3000',
          vault_id: '0x5555555555555555555555555555555555555555555555555555555555555555',
        },
      ],
      inputs: [
        {
          id: 'vault6',
          token: {
            id: 'token6',
            address: '0xffffff6666666666666666666666666666666666',
            name: 'Token6',
            symbol: 'TK6',
            decimals: '18',
          },
          balance: '2500',
          vault_id: '0x6666666666666666666666666666666666666666666666666666666666666666',
        },
      ],
      active: true,
      add_events: [
        {
          transaction: {
            block_number: '12347',
            timestamp: '1620000020',
          },
        },
      ],
      meta: 'metadata3',
      timestamp_added: '1620000020',
    },
    rainlang: 'rainlang3',
  },
];

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the order_detail command correctly', async () => {
    mockIPC((cmd) => {
      if (cmd === 'order_detail') {
        return mockOrderDetailsExtended;
      }
    });

    // check for a result with no URL
    expect(await orderDetail('1', undefined)).toEqual(undefined);

    // check for a result with a URL
    expect(await orderDetail('1', 'http://localhost:8000')).toEqual(mockOrderDetailsExtended);
  });
}
