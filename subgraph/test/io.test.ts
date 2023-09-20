import assert from "assert";
import { FetchResult } from "apollo-fetch";
import { orderBook, signers, subgraph } from "./0_initialization.test";
import { ReserveToken18 } from "../typechain";
import {
  MemoryType,
  Opcode,
  basicDeploy,
  compareStructs,
  eighteenZeros,
  generateEvaluableConfig,
  max_uint256,
  memoryOperand,
  op,
  randomUint256,
} from "../utils";
import { ethers } from "hardhat";
import { encodeMeta } from "../utils/orderBook/order";
import { concat } from "ethers/lib/utils";
import {
  AddOrderEvent,
  OrderConfigStruct,
} from "../typechain/contracts/orderbook/OrderBook";
import { getEventArgs, waitForSubgraphToBeSynced } from "./utils";

describe("IO entity", () => {
  let tokenA: ReserveToken18;
  let tokenB: ReserveToken18;

  beforeEach(async () => {
    tokenA = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    tokenB = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
  });

  it("should query the IO entities of Inputs after adding an order", async () => {
    const [, alice] = signers;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const aliceOrder = encodeMeta("Order_A");

    // Order_A
    const ratio_A = ethers.BigNumber.from("90" + eighteenZeros);
    const constants_A = [max_uint256, ratio_A];
    const aOpMax = op(
      Opcode.read_memory,
      memoryOperand(MemoryType.Constant, 0)
    );
    const aRatio = op(
      Opcode.read_memory,
      memoryOperand(MemoryType.Constant, 1)
    );
    // prettier-ignore
    const source_A = concat([
      aOpMax,
      aRatio,
    ]);

    const EvaluableConfig_A = await generateEvaluableConfig(
      [source_A, []],
      constants_A
    );

    const orderConfig_A: OrderConfigStruct = {
      validInputs: [
        { token: tokenA.address, decimals: 18, vaultId: aliceInputVault },
      ],
      validOutputs: [
        { token: tokenB.address, decimals: 18, vaultId: aliceOutputVault },
      ],
      evaluableConfig: EvaluableConfig_A,
      meta: aliceOrder,
    };

    const txOrder_A = await orderBook.connect(alice).addOrder(orderConfig_A);

    const {
      sender: sender_A,
      expressionDeployer: ExpressionDeployer_A,
      order: order_A,
      orderHash: orderHash_A,
    } = (await getEventArgs(
      txOrder_A,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(
      ExpressionDeployer_A === EvaluableConfig_A.deployer,
      "wrong expression deployer"
    );
    assert(sender_A === alice.address, "wrong sender");
    compareStructs(order_A, orderConfig_A);

    await waitForSubgraphToBeSynced();

    // Subgraph check
    const orderOwner = order_A.owner.toLowerCase();
    const orderHash = orderHash_A.toHexString().toLowerCase();

    for (let i = 0; i < order_A.validInputs.length; i++) {
      const IOValue = order_A.validInputs[i];
      const IOToken = IOValue.token.toLowerCase();
      const IOVault = IOValue.vaultId;
      const IODecimals = IOValue.decimals.toString();

      // ID = `{order.hash()}-{IO.token}-{IO.vaultId}`
      const IO_ID = `${orderHash}-${IOToken}-${IOVault}`;
      // ID = `{vaultId}-{owner}`
      const vault_ID = `${IOVault}-${orderOwner}`;

      const query = `{
        io (id: "${IO_ID}") {
          decimals
          token {
            id
          }
          vault {
            id
          }
          order {
            id
          }
        }
      }`;

      const response = (await subgraph({ query })) as FetchResult;
      const data = response.data.io;

      assert.equal(data.decimals, IODecimals, "wrond decimals on IO");
      assert.equal(data.token.id, IOToken, "wrong token ID on IO");
      assert.equal(data.vault.id, vault_ID, "wrong vault ID on IO");
      assert.equal(data.order.id, orderHash, "wrong order ID on IO");
    }
  });

  it("should query the IO entities of Outputs after adding an order", async () => {
    const [, alice] = signers;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const aliceOrder = encodeMeta("Order_A");

    // Order_A
    const ratio_A = ethers.BigNumber.from("90" + eighteenZeros);
    const constants_A = [max_uint256, ratio_A];
    const aOpMax = op(
      Opcode.read_memory,
      memoryOperand(MemoryType.Constant, 0)
    );
    const aRatio = op(
      Opcode.read_memory,
      memoryOperand(MemoryType.Constant, 1)
    );
    // prettier-ignore
    const source_A = concat([
      aOpMax,
      aRatio,
    ]);

    const EvaluableConfig_A = await generateEvaluableConfig(
      [source_A, []],
      constants_A
    );

    const orderConfig_A: OrderConfigStruct = {
      validInputs: [
        { token: tokenA.address, decimals: 18, vaultId: aliceInputVault },
      ],
      validOutputs: [
        { token: tokenB.address, decimals: 18, vaultId: aliceOutputVault },
      ],
      evaluableConfig: EvaluableConfig_A,
      meta: aliceOrder,
    };

    const txOrder_A = await orderBook.connect(alice).addOrder(orderConfig_A);

    const {
      sender: sender_A,
      expressionDeployer: ExpressionDeployer_A,
      order: order_A,
      orderHash: orderHash_A,
    } = (await getEventArgs(
      txOrder_A,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(
      ExpressionDeployer_A === EvaluableConfig_A.deployer,
      "wrong expression deployer"
    );
    assert(sender_A === alice.address, "wrong sender");
    compareStructs(order_A, orderConfig_A);

    await waitForSubgraphToBeSynced();

    // Subgraph check
    const orderOwner = order_A.owner.toLowerCase();
    const orderHash = orderHash_A.toHexString().toLowerCase();

    for (let i = 0; i < order_A.validOutputs.length; i++) {
      const IOValue = order_A.validOutputs[i];
      const IOToken = IOValue.token.toLowerCase();
      const IOVault = IOValue.vaultId;
      const IODecimals = IOValue.decimals.toString();

      // ID = `{order.hash()}-{IO.token}-{IO.vaultId}`
      const IO_ID = `${orderHash}-${IOToken}-${IOVault}`;
      // ID = `{vaultId}-{owner}`
      const vault_ID = `${IOVault}-${orderOwner}`;

      const query = `{
        io (id: "${IO_ID}") {
          decimals
          token {
            id
          }
          vault {
            id
          }
          order {
            id
          }
        }
      }`;

      const response = (await subgraph({ query })) as FetchResult;
      const data = response.data.io;

      assert.equal(data.decimals, IODecimals, "wrond decimals on IO");
      assert.equal(data.token.id, IOToken, "wrong token ID on IO");
      assert.equal(data.vault.id, vault_ID, "wrong vault ID on IO");
      assert.equal(data.order.id, orderHash, "wrong order ID on IO");
    }
  });
});
