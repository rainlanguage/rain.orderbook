import assert from "assert";
import { getLocal } from "mockttp";
import { describe, it, beforeEach, afterEach } from "vitest";
import {
  Order,
  OrderWithSubgraphName,
  Trade
} from "../../dist/types/js_api.js";
import {
  getOrders,
  getOrder,
  getOrderTradesList,
  getOrderTradeDetail,
  
} from "../../dist/cjs/js_api.js";

const order1 = {
  id: "order1",
  orderBytes:
    "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
  orderHash: "0x1",
  owner: "0x0000000000000000000000000000000000000000",
  outputs: [
    {
      id: "0x0000000000000000000000000000000000000000",
      token: {
        id: "0x0000000000000000000000000000000000000000",
        address: "0x0000000000000000000000000000000000000000",
        name: "T1",
        symbol: "T1",
        decimals: "0",
      },
      balance: "0",
      vaultId: "0",
      owner: "0x0000000000000000000000000000000000000000",
      ordersAsOutput: [],
      ordersAsInput: [],
      balanceChanges: [],
      orderbook: {
        id: "0x0000000000000000000000000000000000000000",
      },
    },
  ],
  inputs: [
    {
      id: "0x0000000000000000000000000000000000000000",
      token: {
        id: "0x0000000000000000000000000000000000000000",
        address: "0x0000000000000000000000000000000000000000",
        name: "T2",
        symbol: "T2",
        decimals: "0",
      },
      balance: "0",
      vaultId: "0",
      owner: "0x0000000000000000000000000000000000000000",
      ordersAsOutput: [],
      ordersAsInput: [],
      balanceChanges: [],
      orderbook: {
        id: "0x0000000000000000000000000000000000000000",
      },
    },
  ],
  active: true,
  addEvents: [
    {
      transaction: {
        blockNumber: "0",
        timestamp: "0",
        id: "0x0000000000000000000000000000000000000000",
        from: "0x0000000000000000000000000000000000000000",
      },
    },
  ],
  meta: null,
  timestampAdded: "0",
  orderbook: {
    id: "0x0000000000000000000000000000000000000000",
  },
  trades: [],
};
const order2 = {
  id: "order2",
  orderBytes:
    "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
  orderHash: "0x2",
  owner: "0x0000000000000000000000000000000000000000",
  outputs: [
    {
      id: "0x0000000000000000000000000000000000000000",
      token: {
        id: "0x0000000000000000000000000000000000000000",
        address: "0x0000000000000000000000000000000000000000",
        name: "T1",
        symbol: "T1",
        decimals: "0",
      },
      balance: "0",
      vaultId: "0",
      owner: "0x0000000000000000000000000000000000000000",
      ordersAsOutput: [],
      ordersAsInput: [],
      balanceChanges: [],
      orderbook: {
        id: "0x0000000000000000000000000000000000000000",
      },
    },
  ],
  inputs: [
    {
      id: "0x0000000000000000000000000000000000000000",
      token: {
        id: "0x0000000000000000000000000000000000000000",
        address: "0x0000000000000000000000000000000000000000",
        name: "T2",
        symbol: "T2",
        decimals: "0",
      },
      balance: "0",
      vaultId: "0",
      owner: "0x0000000000000000000000000000000000000000",
      ordersAsOutput: [],
      ordersAsInput: [],
      balanceChanges: [],
      orderbook: {
        id: "0x0000000000000000000000000000000000000000",
      },
    },
  ],
  active: true,
  addEvents: [
    {
      transaction: {
        blockNumber: "0",
        timestamp: "0",
        id: "0x0000000000000000000000000000000000000000",
        from: "0x0000000000000000000000000000000000000000",
      },
    },
  ],
  meta: null,
  timestampAdded: "0",
  orderbook: {
    id: "0x0000000000000000000000000000000000000000",
  },
  trades: [],
};


const mockTradeOrdersList = [
  {
    id: "0x07db8b3f3e7498f9d4d0e40b98f57c020d3d277516e86023a8200a20464d4894",
    timestamp: "1632000000",
    tradeEvent: {
      sender: "0x0000000000000000000000000000000000000000",
      transaction: {
        id: "0x0000000000000000000000000000000000000000",
        from: "0x0000000000000000000000000000000000000000",
        timestamp: "1632000000",
        blockNumber: "0",
      },
    },
    outputVaultBalanceChange: {
      amount: "-100",
      vault: {
        id: "vault-1",
        vaultId: "1",
        token: {
          id: "token-1",
          address: "0x1111111111111111111111111111111111111111",
          name: "Token One",
          symbol: "TK1",
          decimals: "18",
        },
      },
      id: "output-change-1",
      __typename: "TradeVaultBalanceChange",
      newVaultBalance: "900",
      oldVaultBalance: "1000",
      timestamp: "1632000000",
      transaction: {
        id: "0x0000000000000000000000000000000000000000",
        from: "0x0000000000000000000000000000000000000000",
        timestamp: "1632000000",
        blockNumber: "0",
      },
      orderbook: { id: "orderbook-1" },
    },
    order: {
      id: order1.id,
      orderHash:
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    },
    inputVaultBalanceChange: {
      amount: "50",
      vault: {
        id: "vault-2",
        vaultId: "2",
        token: {
          id: "token-2",
          address: "0x2222222222222222222222222222222222222222",
          name: "Token Two",
          symbol: "TK2",
          decimals: "18",
        },
      },
      id: "input-change-1",
      __typename: "TradeVaultBalanceChange",
      newVaultBalance: "150",
      oldVaultBalance: "100",
      timestamp: "1632000000",
      transaction: {
        id: "0x0000000000000000000000000000000000000000",
        from: "0x0000000000000000000000000000000000000000",
        timestamp: "1632000000",
        blockNumber: "0",
      },
      orderbook: { id: "orderbook-1" },
    },
    orderbook: {
      id: "orderbook-1",
    },
  },
];

const mockTrade = {
  id: "trade1",
  order: {
    id: "order1",
    orderHash: "0x1",
  },
  tradeEvent: {
    sender: "0x0000000000000000000000000000000000000000",
    transaction: {
      id: "0x0000000000000000000000000000000000000000",
      from: "0x0000000000000000000000000000000000000000",
      blockNumber: "0",
      timestamp: "0",
    },
  },
  timestamp: "0",
  orderbook: {
    id: "0x0000000000000000000000000000000000000000",
  },
  outputVaultBalanceChange: {
    id: "0x0000000000000000000000000000000000000000",
    __typename: "TradeVaultBalanceChange",
    amount: "-7",
    newVaultBalance: "93",
    oldVaultBalance: "100",
    vault: {
      id: "0x0000000000000000000000000000000000000000",
      vaultId: "1",
      token: {
        id: "0x0000000000000000000000000000000000000000",
        address: "0x0000000000000000000000000000000000000000",
        name: "T1",
        symbol: "T1",
        decimals: "18",
      },
    },
    timestamp: "0",
    transaction: {
      id: "0x0000000000000000000000000000000000000000",
      from: "0x0000000000000000000000000000000000000000",
      blockNumber: "0",
      timestamp: "0",
    },
    orderbook: {
      id: "0x0000000000000000000000000000000000000000",
    },
  },
  inputVaultBalanceChange: {
    id: "0x0000000000000000000000000000000000000000",
    __typename: "TradeVaultBalanceChange",
    amount: "5",
    newVaultBalance: "105",
    oldVaultBalance: "100",
    vault: {
      id: "0x0000000000000000000000000000000000000000",
      vaultId: "2",
      token: {
        id: "0x0000000000000000000000000000000000000000",
        address: "0x0000000000000000000000000000000000000000",
        name: "T2",
        symbol: "T2",
        decimals: "6",
      },
    },
    timestamp: "0",
    transaction: {
      id: "0x0000000000000000000000000000000000000000",
      from: "0x0000000000000000000000000000000000000000",
      blockNumber: "0",
      timestamp: "0",
    },
    orderbook: {
      id: "0x0000000000000000000000000000000000000000",
    },
  },
};

describe("Rain Orderbook JS API Package Bindgen Tests - Order", async function () {
  const mockServer = getLocal();
  beforeEach(() => mockServer.start(8082));
  afterEach(() => mockServer.stop());

  it("should fetch a single order", async () => {
    await mockServer
      .forPost("/sg1")
      .thenReply(200, JSON.stringify({ data: { order: order1 } }));

    try {
      const result: Order = await getOrder(mockServer.url + "/sg1", order1.id);
      assert.equal(result.id, order1.id);
    } catch (e) {
      console.log(e);
      assert.fail("expected to resolve, but failed" + (e instanceof Error ? e.message : String(e)));
    }
  });

  it("should fetch multiple orders from different subgraphs", async () => {
    await mockServer
      .forPost("/sg1")
      .thenReply(200, JSON.stringify({ data: { orders: [order1] } }));
    await mockServer
      .forPost("/sg2")
      .thenReply(200, JSON.stringify({ data: { orders: [order2] } }));

    try {
      const result: OrderWithSubgraphName[] = await getOrders(
        [
          { url: mockServer.url + "/sg1", name: "network-one" },
          { url: mockServer.url + "/sg2", name: "network-two" },
        ],
        {
          owners: [],
          active: undefined,
          orderHash: undefined,
        },
        {
          page: 1,
          pageSize: 10,
        }
      );
      assert.equal(result.length, 2);
      assert.equal(result[0].order.id, order1.id);
      assert.equal(result[0].subgraphName, "network-one");
      assert.equal(result[1].order.id, order2.id);
      assert.equal(result[1].subgraphName, "network-two");
    } catch (e) {
      console.log(e);
      assert.fail("expected to resolve, but failed" + (e instanceof Error ? e.message : String(e)));
    }
  });

  it("should fetch trades for a single order", async () => {
    // Mock server response for trades - wrap the trades in the expected structure
    await mockServer.forPost("/sg1").thenReply(
      200,
      JSON.stringify({
        data: {
          trades: mockTradeOrdersList,
        },
      })
    );

    try {
      const result = await getOrderTradesList(
        mockServer.url + "/sg1",
        order1.id,
        {
          page: 1,
          pageSize: 10,
        },
        undefined,
        undefined
      );

      console.log("RESULT!", result);

      assert.ok(result, "Result should exist");
      assert.equal(result.length, 1, "Should have one trade");
      assert.equal(
        result[0].id,
        "0x07db8b3f3e7498f9d4d0e40b98f57c020d3d277516e86023a8200a20464d4894",
        "Trade ID should match"
      );
    } catch (e: unknown) {
      console.error("Test error:", e);
      assert.fail("Expected to resolve, but failed: " + (e instanceof Error ? e.message : String(e)));
    }
  });

  it("should fetch order trade detail", async () => {
    await mockServer
      .forPost("/sg1")
      .thenReply(200, JSON.stringify({ data: { trade: mockTrade } }));

    try {
      const result: Trade = await getOrderTradeDetail(
        mockServer.url + "/sg1",
        mockTrade.id
      );
      assert.equal(result.id, mockTrade.id);
      assert.equal(result.order.id, mockTrade.order.id);
      assert.equal(
        result.outputVaultBalanceChange.amount,
        mockTrade.outputVaultBalanceChange.amount
      );
      assert.equal(
        result.inputVaultBalanceChange.amount,
        mockTrade.inputVaultBalanceChange.amount
      );
    } catch (e) {
      console.log(e);
      assert.fail("expected to resolve, but failed" + + (e instanceof Error ? e.message : String(e)));
    }
  });
});
