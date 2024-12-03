import type { OrderDetailExtended } from "../typeshare/subgraphTypes";

export const mockOrderDetailsExtended: OrderDetailExtended = {
  order: {
    id: 'order1',
    orderBytes: '0x123456',
    orderHash: '0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef',
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
        vaultId: '0x1111111111111111111111111111111111111111111111111111111111111111',
        orderbook: { id: '0x1111111111111111111111111111111111111111' },
        owner: '0x1111111111111111111111111111111111111111',
        ordersAsOutput: [],
        ordersAsInput: [],
        balanceChanges: [],
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
        vaultId: '0x2222222222222222222222222222222222222222222222222222222222222222',
        orderbook: { id: '0x1111111111111111111111111111111111111111' },
        owner: '0x1111111111111111111111111111111111111111',
        ordersAsOutput: [],
        ordersAsInput: [],
        balanceChanges: [],
      },
    ],
    active: true,
    addEvents: [
      {
        transaction: {
          id: '0x2222222222222222222222222222222222222222222222222222222222222222',
          from: '0x1111111111111111111111111111111111111111',
          blockNumber: '12345',
          timestamp: '1620000000',
        },
      },
    ],
    meta: 'metadata1',
    timestampAdded: '1620000000',
    orderbook: {
      id: '0x00',
    },
    trades: [],
  },
  rainlang: 'rainlang1',
};