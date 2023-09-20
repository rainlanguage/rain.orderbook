import {
  Bounty,
  IO,
  MetaContentV1,
  Order,
  OrderClearStateChange,
  ClearOrderConfig,
  TokenVault,
  ContextEntity,
  TokenVaultTakeOrder,
  OrderBook,
} from "../generated/schema";
import {
  AddOrder,
  AfterClear,
  Clear,
  Context,
  Deposit,
  MetaV1,
  OrderExceedsMaxRatio,
  OrderNotFound,
  OrderZeroAmount,
  RemoveOrder,
  TakeOrder,
  Withdraw,
  // Initialized,
  ClearAliceStruct,
} from "../generated/OrderBook/OrderBook";
import {
  BigDecimal,
  BigInt,
  Bytes,
  JSONValue,
  JSONValueKind,
  TypedMap,
  // ValueKind,
  ethereum,
  json,
  // log,
  // store,
} from "@graphprotocol/graph-ts";

import {
  AFTER_CLEAR_EVENT_TOPIC,
  CLEAR_EVENT_TOPIC,
  NEW_EXPRESSION_EVENT_TOPIC,
  RAIN_META_DOCUMENT_HEX,
  TAKE_ORDER_EVENT_TOPIC,
  createAccount,
  createContextEntity,
  createOrder,
  createOrderClear,
  createSignedContext,
  createTakeOrderConfig,
  createToken,
  createTokenVault,
  createTransaction,
  createVault,
  createVaultDeposit,
  createVaultWithdraw,
  getEvenHex,
  getKeccak256FromBytes,
  // getOB,
  getRainMetaV1,
  isHexadecimalString,
  stringToArrayBuffer,
  toDisplay,
  tuplePrefix,
} from "./utils";
import { CBORDecoder } from "@rainprotocol/assemblyscript-cbor";
import { ExpressionJSONString, OrderString } from "./orderJsonString";

export function handleContext(event: Context): void {
  const receipt = event.receipt;

  // Should be at least one log (the current event is one). This is conditional
  // is just for safe typing.
  if (receipt && receipt.logs.length > 0) {
    const logs = receipt.logs;

    const log_takeOrder = logs.findIndex(
      (log_) => log_.topics[0].toHex() == TAKE_ORDER_EVENT_TOPIC
    );
    const log_clear = logs.findIndex(
      (log_) => log_.topics[0].toHex() == CLEAR_EVENT_TOPIC
    );
    const log_afterClear = logs.findIndex(
      (log_) => log_.topics[0].toHex() == AFTER_CLEAR_EVENT_TOPIC
    );

    if (log_clear != -1 && log_afterClear != -1) {
      // It's a context for clear and afterClear
      // This is in case that need to support context for these events
    }

    if (log_takeOrder != -1) {
      // It's a context for a takeOrder
      // Create the TakeOrder and assign the context entity ID.
      // ON the takeOrder handler, the takeOrder entity should not be created, only
      // read/obtained and modified with the data of the take order event.
      const context = event.params.context;

      // Column 0 is the Context Base
      const sender = Bytes.fromHexString(getEvenHex(context[0][0].toHex()));
      const callerContract = Bytes.fromHexString(
        getEvenHex(context[0][1].toHex())
      );

      // Column 1 is the Context Calling
      const callingContext = context[1];

      // Column 2 is the Context Calculations
      const calculationsContext = context[2];

      // Column 3 is the Context Vault Inputs
      const vaultInputsContext = context[3];

      // Column 4 is the Context Vault Outputs
      const vaultOutputsContext = context[4];

      // After the column 4, all is of signedContext
      const signedContextArr = context.slice(5);

      // Where all the "real" signed conext entities are "save"
      const signedContextEntities: string[] = [];

      if (signedContextArr.length > 0) {
        const signers = signedContextArr.shift();

        if (signers.length == signedContextArr.length) {
          for (let i = 0; i < signedContextArr.length; i++) {
            const signedContextEntity = createSignedContext(
              event.transaction.hash.toHex(),
              event.logIndex.toHex()
            );

            signedContextEntity.context = signedContextArr[i];
            signedContextEntity.signer = Bytes.fromHexString(
              getEvenHex(signers[i].toHex())
            );

            signedContextEntity.save();

            signedContextEntities.push(signedContextEntity.id);
          }
        }
      }

      // Temp
      const contextTakeOrder = new ContextEntity("ContextTakeOrderTemp");

      contextTakeOrder.caller = sender;
      contextTakeOrder.contract = callerContract;
      contextTakeOrder.callingContext = callingContext;
      contextTakeOrder.calculationsContext = calculationsContext;
      contextTakeOrder.vaultInputsContext = vaultInputsContext;
      contextTakeOrder.vaultOutputsContext = vaultOutputsContext;
      contextTakeOrder.signedContext = signedContextEntities;

      contextTakeOrder.emitter = createAccount(event.params.sender).id;
      contextTakeOrder.timestamp = event.block.timestamp;
      contextTakeOrder.transaction = createTransaction(
        event.transaction.hash.toHex(),
        event.block
      ).id;

      contextTakeOrder.save();
    }
  }
}

export function handleAddOrder(event: AddOrder): void {
  // Order parameter from event
  const orderParam = event.params.order;

  const orderHashHex = getEvenHex(event.params.orderHash.toHex());

  let order = new Order(orderHashHex);

  order.ordersClears = [];
  order.transaction = createTransaction(
    event.transaction.hash.toHex(),
    event.block
  ).id;

  order.orderHash = Bytes.fromHexString(
    getEvenHex(event.params.orderHash.toHex())
  );
  order.timestamp = event.block.timestamp;
  order.owner = createAccount(orderParam.owner).id;
  order.emitter = createAccount(event.params.sender).id;

  order.expressionDeployer = event.params.expressionDeployer;
  order.expression = orderParam.evaluable.expression;
  order.interpreter = orderParam.evaluable.interpreter;
  order.interpreterStore = orderParam.evaluable.store;
  order.handleIO = orderParam.handleIO;
  order.orderActive = true;
  order.validInputs = [];
  order.validOutputs = [];

  for (let i = 0; i < orderParam.validInputs.length; i++) {
    let token = createToken(orderParam.validInputs[i].token);
    let vault = createVault(
      orderParam.validInputs[i].vaultId.toString(),
      orderParam.owner
    );
    let tokenVault = createTokenVault(
      orderParam.validInputs[i].vaultId.toString(),
      event.params.sender,
      orderParam.validInputs[i].token
    );

    if (tokenVault) {
      let orders = tokenVault.orders;
      if (orders) orders.push(order.id);
      tokenVault.orders = orders;
      tokenVault.save();
    }

    let input = new IO(
      `${orderHashHex}-${token.id}-${orderParam.validInputs[i].vaultId}`
    );
    input.token = token.id;
    input.decimals = orderParam.validInputs[i].decimals;
    input.vault = vault.id;
    input.vaultId = orderParam.validInputs[i].vaultId;
    input.order = orderHashHex;
    input.tokenVault = tokenVault.id;
    input.index = i;
    input.save();

    // Add the input to the order entity
    const auxInput = order.validInputs;
    if (auxInput) if (!auxInput.includes(input.id)) auxInput.push(input.id);
    order.validInputs = auxInput;
  }

  for (let i = 0; i < orderParam.validOutputs.length; i++) {
    let token = createToken(orderParam.validOutputs[i].token);
    let vault = createVault(
      orderParam.validOutputs[i].vaultId.toString(),
      orderParam.owner
    );
    let tokenVault = createTokenVault(
      orderParam.validOutputs[i].vaultId.toString(),
      event.params.sender,
      orderParam.validOutputs[i].token
    );

    if (tokenVault) {
      let orders = tokenVault.orders;
      if (orders) orders.push(order.id);
      tokenVault.orders = orders;
      tokenVault.save();
    }

    let output = new IO(
      `${orderHashHex}-${token.id}-${orderParam.validOutputs[i].vaultId}`
    );
    output.token = token.id;
    output.decimals = orderParam.validOutputs[i].decimals;
    output.vault = vault.id;
    output.vaultId = orderParam.validOutputs[i].vaultId;
    output.order = orderHashHex;
    output.tokenVault = tokenVault.id;
    output.index = i;
    output.save();

    // Use the OrderString class to generate a Order JSON string compatible value
    const orderString = new OrderString(orderParam);
    order.orderJSONString = orderString.stringify();

    // Add the input to the order entity
    const auxOutput = order.validOutputs;
    if (auxOutput)
      if (!auxOutput.includes(output.id)) auxOutput.push(output.id);

    order.validOutputs = auxOutput;
  }

  const receipt = event.receipt;
  if (receipt && receipt.logs.length > 0) {
    const logs = receipt.logs;

    const log_newExpression = logs.findIndex(
      (log_) => log_.topics[0].toHex() == NEW_EXPRESSION_EVENT_TOPIC
    );

    if (log_newExpression != -1) {
      const log_callerMeta = logs[log_newExpression];

      const dataTuple = tuplePrefix.concat(log_callerMeta.data);

      const decodedData = ethereum.decode(
        "(address,bytes[],uint256[],uint256[])",
        dataTuple
      );

      if (decodedData && decodedData.kind === ethereum.ValueKind.TUPLE) {
        const newExpressionTuple = decodedData.toTuple();

        const sources_ = newExpressionTuple[1].toBytesArray();
        const constants_ = newExpressionTuple[2].toBigIntArray();

        const expressionJsonString = new ExpressionJSONString(
          sources_,
          constants_
        );
        order.expressionJSONString = expressionJsonString.stringify();
      }
    }
  }

  order.save();
}

export function handleAfterClear(event: AfterClear): void {
  let config = ClearOrderConfig.load("1");
  const clearStateChange = event.params.clearStateChange;

  // Amounts
  const aliceInput = clearStateChange.aliceInput;
  const aliceOutput = clearStateChange.aliceOutput;
  const bobInput = clearStateChange.bobInput;
  const bobOutput = clearStateChange.bobOutput;

  // Bounty amounts
  const bountyAmountA = aliceOutput.minus(bobInput);
  const bountyAmountB = bobOutput.minus(aliceInput);

  if (config) {
    let orderClearStateChange = new OrderClearStateChange(config.orderClearId);
    orderClearStateChange.orderClear = config.orderClearId;
    orderClearStateChange.aInput = aliceInput;
    orderClearStateChange.aOutput = aliceOutput;
    orderClearStateChange.bInput = bobInput;
    orderClearStateChange.bOutput = bobOutput;
    orderClearStateChange.save();

    let bounty = Bounty.load(config.orderClearId);
    if (bounty) {
      bounty.bountyAmountA = bountyAmountA;
      bounty.bountyAmountADisplay = toDisplay(
        bountyAmountA,
        bounty.bountyTokenA
      );
      bounty.bountyAmountB = bountyAmountB;
      bounty.bountyAmountBDisplay = toDisplay(
        bountyAmountB,
        bounty.bountyTokenB
      );
      bounty.save();
    }

    if (bountyAmountA.gt(BigInt.zero())) {
      const tokenVaultBounty_A = TokenVault.load(config.tokenVaultBountyAlice);

      if (tokenVaultBounty_A) {
        tokenVaultBounty_A.balance =
          tokenVaultBounty_A.balance.plus(bountyAmountA);
        tokenVaultBounty_A.balanceDisplay = toDisplay(
          tokenVaultBounty_A.balance,
          tokenVaultBounty_A.token
        );

        tokenVaultBounty_A.save();
      }
    }

    if (bountyAmountB.gt(BigInt.zero())) {
      const tokenVaultBounty_B = TokenVault.load(config.tokenVaultBountyBob);

      if (tokenVaultBounty_B) {
        tokenVaultBounty_B.balance =
          tokenVaultBounty_B.balance.plus(bountyAmountB);
        tokenVaultBounty_B.balanceDisplay = toDisplay(
          tokenVaultBounty_B.balance,
          tokenVaultBounty_B.token
        );

        tokenVaultBounty_B.save();
      }
    }

    // TokenVaults IDs to update balance
    const aliceTokenVaultInput_ID = config.aliceTokenVaultInput;
    const aliceTokenVaultOutput_ID = config.aliceTokenVaultOutput;
    const bobTokenVaultInput_ID = config.bobTokenVaultInput;
    const bobTokenVaultOutput_ID = config.bobTokenVaultOutput;

    // Updating alice input/output balance
    const aliceTokenVaultInput = TokenVault.load(aliceTokenVaultInput_ID);
    if (aliceTokenVaultInput) {
      aliceTokenVaultInput.balance =
        aliceTokenVaultInput.balance.plus(aliceInput);
      aliceTokenVaultInput.balanceDisplay = toDisplay(
        aliceTokenVaultInput.balance,
        aliceTokenVaultInput.token
      );
      aliceTokenVaultInput.save();
    }

    const aliceTokenVaultOutput = TokenVault.load(aliceTokenVaultOutput_ID);
    if (aliceTokenVaultOutput) {
      aliceTokenVaultOutput.balance =
        aliceTokenVaultOutput.balance.minus(aliceOutput);
      aliceTokenVaultOutput.balanceDisplay = toDisplay(
        aliceTokenVaultOutput.balance,
        aliceTokenVaultOutput.token
      );
      aliceTokenVaultOutput.save();
    }

    // Updating bob input/output balance
    const bobTokenVaultInput = TokenVault.load(bobTokenVaultInput_ID);
    if (bobTokenVaultInput) {
      bobTokenVaultInput.balance = bobTokenVaultInput.balance.plus(bobInput);
      bobTokenVaultInput.balanceDisplay = toDisplay(
        bobTokenVaultInput.balance,
        bobTokenVaultInput.token
      );
      bobTokenVaultInput.save();
    }

    const bobTokenVaultOutput = TokenVault.load(bobTokenVaultOutput_ID);
    if (bobTokenVaultOutput) {
      bobTokenVaultOutput.balance =
        bobTokenVaultOutput.balance.minus(bobOutput);
      bobTokenVaultOutput.balanceDisplay = toDisplay(
        bobTokenVaultOutput.balance,
        bobTokenVaultOutput.token
      );
      bobTokenVaultOutput.save();
    }
  }
}

export function handleClear(event: Clear): void {
  const alice = event.params.alice;
  const bob = event.params.bob;
  const clearConfig = event.params.clearConfig;
  const sender = event.params.sender;

  let orderClear = createOrderClear(event.transaction.hash.toHex());
  orderClear.sender = createAccount(sender).id;
  orderClear.clearer = createAccount(sender).id;

  const order_A = createOrder(alice);
  const order_B = createOrder(changetype<ClearAliceStruct>(bob));

  orderClear.orderA = order_A.id;
  orderClear.orderB = order_B.id;

  // Saving order clears to each orders
  // - ORDER A
  const ordersClears_A = order_A.ordersClears;
  if (ordersClears_A) ordersClears_A.push(orderClear.id);
  order_A.ordersClears = ordersClears_A;
  order_A.save();
  // - ORDER B
  const ordersClears_B = order_B.ordersClears;
  if (ordersClears_B) ordersClears_B.push(orderClear.id);
  order_B.ordersClears = ordersClears_B;
  order_B.save();

  orderClear.owners = [
    createAccount(alice.owner).id,
    createAccount(bob.owner).id,
  ];
  orderClear.aInputIOIndex = clearConfig.aliceInputIOIndex;
  orderClear.aOutputIOIndex = clearConfig.aliceOutputIOIndex;
  orderClear.bInputIOIndex = clearConfig.bobInputIOIndex;
  orderClear.bOutputIOIndex = clearConfig.bobOutputIOIndex;
  orderClear.transaction = createTransaction(
    event.transaction.hash.toHex(),
    event.block
  ).id;
  orderClear.emitter = createAccount(event.params.sender).id;
  orderClear.timestamp = event.block.timestamp;
  orderClear.save();

  let bounty = new Bounty(orderClear.id);
  bounty.clearer = createAccount(sender).id;
  bounty.orderClear = orderClear.id;
  bounty.bountyVaultA = createVault(
    clearConfig.aliceBountyVaultId.toString(),
    sender
  ).id;
  bounty.bountyVaultB = createVault(
    clearConfig.bobBountyVaultId.toString(),
    sender
  ).id;

  bounty.bountyTokenA = createToken(
    alice.validOutputs[clearConfig.aliceOutputIOIndex.toI32()].token
  ).id;
  bounty.bountyTokenB = createToken(
    bob.validOutputs[clearConfig.bobOutputIOIndex.toI32()].token
  ).id;
  bounty.transaction = createTransaction(
    event.transaction.hash.toHex(),
    event.block
  ).id;
  bounty.emitter = createAccount(event.params.sender).id;
  bounty.timestamp = event.block.timestamp;
  bounty.save();

  // IO Index values used to clear (for alice and bob)
  const aliceInputIOIndex = clearConfig.aliceInputIOIndex.toI32();
  const aliceOutputIOIndex = clearConfig.aliceOutputIOIndex.toI32();
  const bobInputIOIndex = clearConfig.bobInputIOIndex.toI32();
  const bobOutputIOIndex = clearConfig.bobOutputIOIndex.toI32();

  // Valid inputs/outpus based on the Index used (for alice and bob)
  const aliceInputValues = alice.validInputs[aliceInputIOIndex];
  const aliceOutputValues = alice.validOutputs[aliceOutputIOIndex];
  const bobInputValues = bob.validInputs[bobInputIOIndex];
  const bobOutputValues = bob.validOutputs[bobOutputIOIndex];

  // Token input/output based on the Index used (for alice and bob)
  const aliceTokenInput = aliceInputValues.token;
  const aliceTokenOutput = aliceOutputValues.token;
  const bobTokenInput = bobInputValues.token;
  const bobTokenOutput = bobOutputValues.token;

  // Vault IDs input/output based on the Index used (for alice and bob)
  const aliceVaultInput = aliceInputValues.vaultId;
  const aliceVaultOutput = aliceOutputValues.vaultId;
  const bobVaultInput = bobInputValues.vaultId;
  const bobVaultOutput = bobOutputValues.vaultId;

  // Token Vault IDs to use
  const aliceTokenVaultInput = `${aliceVaultInput.toString()}-${alice.owner.toHex()}-${aliceTokenInput.toHex()}`;
  const aliceTokenVaultOutput = `${aliceVaultOutput.toString()}-${alice.owner.toHex()}-${aliceTokenOutput.toHex()}`;
  const bobTokenVaultInput = `${bobVaultInput.toString()}-${bob.owner.toHex()}-${bobTokenInput.toHex()}`;
  const bobTokenVaultOutput = `${bobVaultOutput.toString()}-${bob.owner.toHex()}-${bobTokenOutput.toHex()}`;

  // Only should link the TokenVault to OrderClear where they are being clear
  // using a the given vault and token.
  // Only will link to four (4) token vaults this clear. Does not care if the
  // orders have more valid inputs/outputs.
  const tokenVaultArr = [
    aliceTokenVaultInput,
    aliceTokenVaultOutput,
    bobTokenVaultInput,
    bobTokenVaultOutput,
  ];

  // The token vault should exist on this point since it was created when
  // the order was added.
  for (let i = 0; i < 4; i++) {
    const tokenVault_ID = tokenVaultArr[i];
    let tokenVault = TokenVault.load(tokenVault_ID);
    if (tokenVault) {
      let orderClears = tokenVault.orderClears;
      if (orderClears) orderClears.push(orderClear.id);
      tokenVault.orderClears = orderClears;
      tokenVault.save();
    }
  }

  // Clearer/Bounty address token vaults
  let tokenVaultBountyAlice = createTokenVault(
    event.params.clearConfig.aliceBountyVaultId.toString(),
    event.params.sender,
    event.params.alice.validOutputs[
      event.params.clearConfig.aliceOutputIOIndex.toI32()
    ].token
  );

  let tokenVaultBountyBob = createTokenVault(
    event.params.clearConfig.bobBountyVaultId.toString(),
    event.params.sender,
    event.params.bob.validOutputs[
      event.params.clearConfig.bobOutputIOIndex.toI32()
    ].token
  );

  let config = new ClearOrderConfig("1");
  config.orderClearId = orderClear.id;
  config.tokenVaultBountyAlice = tokenVaultBountyAlice.id;
  config.tokenVaultBountyBob = tokenVaultBountyBob.id;
  config.aliceTokenVaultInput = aliceTokenVaultInput;
  config.aliceTokenVaultOutput = aliceTokenVaultOutput;
  config.bobTokenVaultInput = bobTokenVaultInput;
  config.bobTokenVaultOutput = bobTokenVaultOutput;
  config.save();
}

export function handleDeposit(event: Deposit): void {
  let tokenVault = createTokenVault(
    event.params.vaultId.toString(),
    event.params.sender,
    event.params.token
  );

  if (tokenVault) {
    tokenVault.balance = tokenVault.balance.plus(event.params.amount);
    tokenVault.balanceDisplay = toDisplay(
      tokenVault.balance,
      event.params.token.toHexString()
    );
    tokenVault.save();
  }

  let vaultDeposit = createVaultDeposit(event.transaction.hash.toHex());
  vaultDeposit.sender = createAccount(event.params.sender).id;
  vaultDeposit.token = createToken(event.params.token).id;
  vaultDeposit.vaultId = event.params.vaultId;
  vaultDeposit.vault = createVault(
    event.params.vaultId.toString(),
    event.params.sender
  ).id;
  vaultDeposit.amount = event.params.amount;
  vaultDeposit.amountDisplay = toDisplay(
    vaultDeposit.amount,
    event.params.token.toHexString()
  );
  vaultDeposit.tokenVault = tokenVault.id;
  vaultDeposit.vault = createVault(
    event.params.vaultId.toString(),
    event.params.sender
  ).id;
  vaultDeposit.transaction = createTransaction(
    event.transaction.hash.toHex(),
    event.block
  ).id;
  vaultDeposit.emitter = createAccount(event.params.sender).id;
  vaultDeposit.timestamp = event.block.timestamp;
  vaultDeposit.save();
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function handleOrderExceedsMaxRatio(event: OrderExceedsMaxRatio): void {
  //
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function handleOrderNotFound(event: OrderNotFound): void {
  //
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function handleOrderZeroAmount(event: OrderZeroAmount): void {
  //
}

export function handleRemoveOrder(event: RemoveOrder): void {
  const orderHashHex = getEvenHex(event.params.orderHash.toHex());

  let order = Order.load(orderHashHex);
  if (order) {
    order.orderActive = false;
    order.save();
  }
}

export function handleTakeOrder(event: TakeOrder): void {
  let takeOrderEntity = createTakeOrderConfig(event.transaction.hash.toHex());
  takeOrderEntity.sender = createAccount(event.params.sender).id;
  takeOrderEntity.order = createOrder(
    changetype<ClearAliceStruct>(event.params.config.order)
  ).id;

  takeOrderEntity.input = event.params.input;
  takeOrderEntity.inputDisplay = toDisplay(
    event.params.input,
    event.params.config.order.validOutputs[
      event.params.config.outputIOIndex.toI32()
    ].token.toHexString()
  );

  takeOrderEntity.output = event.params.output;
  takeOrderEntity.outputDisplay = toDisplay(
    event.params.output,
    event.params.config.order.validInputs[
      event.params.config.inputIOIndex.toI32()
    ].token.toHexString()
  );
  if (takeOrderEntity.outputDisplay != BigDecimal.zero()) {
    takeOrderEntity.IORatio = takeOrderEntity.inputDisplay.div(
      takeOrderEntity.outputDisplay
    );
  } else {
    takeOrderEntity.IORatio = BigDecimal.zero();
  }

  takeOrderEntity.inputIOIndex = event.params.config.inputIOIndex;
  takeOrderEntity.outputIOIndex = event.params.config.outputIOIndex;

  takeOrderEntity.inputToken = createToken(
    event.params.config.order.validOutputs[
      event.params.config.outputIOIndex.toI32()
    ].token
  ).id;

  takeOrderEntity.outputToken = createToken(
    event.params.config.order.validInputs[
      event.params.config.inputIOIndex.toI32()
    ].token
  ).id;

  takeOrderEntity.transaction = createTransaction(
    event.transaction.hash.toHex(),
    event.block
  ).id;
  takeOrderEntity.emitter = createAccount(event.params.sender).id;
  takeOrderEntity.timestamp = event.block.timestamp;

  // Adding the context
  const contextTakeOrder = ContextEntity.load("ContextTakeOrderTemp");
  if (contextTakeOrder) {
    const contextEntity = createContextEntity(
      event.transaction.hash.toHex(),
      event.logIndex.toHex()
    );

    contextEntity.caller = contextTakeOrder.caller;
    contextEntity.contract = contextTakeOrder.contract;
    contextEntity.callingContext = contextTakeOrder.callingContext;
    contextEntity.calculationsContext = contextTakeOrder.calculationsContext;
    contextEntity.vaultInputsContext = contextTakeOrder.vaultInputsContext;
    contextEntity.vaultOutputsContext = contextTakeOrder.vaultOutputsContext;
    contextEntity.signedContext = contextTakeOrder.signedContext;

    contextEntity.emitter = contextTakeOrder.emitter;
    contextEntity.timestamp = contextTakeOrder.timestamp;
    contextEntity.transaction = contextTakeOrder.transaction;

    takeOrderEntity.context = contextEntity.id;

    contextEntity.save();

    // store.remove("ContextEntity", "ContextTakeOrderTemp");
  }

  takeOrderEntity.save();

  // Updating Balance

  let order = event.params.config.order;

  // IO Index values used to takeOrder
  const inputIOIndex = event.params.config.inputIOIndex.toI32();
  const outputIOIndex = event.params.config.outputIOIndex.toI32();

  // Valid inputs/outpus based on the Index used
  const inputValues = order.validInputs[inputIOIndex];
  const outputValues = order.validOutputs[outputIOIndex];

  // Token input/output based on the Index used
  const tokenInput = inputValues.token;
  const tokenOutput = outputValues.token;

  // Vault IDs input/output based on the Index used
  const vaultInput = inputValues.vaultId;
  const vaultOutput = outputValues.vaultId;

  const tokenVaultInput = `${vaultInput.toString()}-${order.owner.toHex()}-${tokenInput.toHex()}`;
  const tokenVaultOutput = `${vaultOutput.toString()}-${order.owner.toHex()}-${tokenOutput.toHex()}`;

  // Updating order input/output balance
  const orderTokenVaultInput = TokenVault.load(tokenVaultInput);
  if (orderTokenVaultInput) {
    orderTokenVaultInput.balance = orderTokenVaultInput.balance.plus(
      event.params.output
    );
    orderTokenVaultInput.balanceDisplay = toDisplay(
      orderTokenVaultInput.balance,
      orderTokenVaultInput.token
    );
    orderTokenVaultInput.save();

    let takeOrderTokenVault = TokenVaultTakeOrder.load(
      `${takeOrderEntity.id}-${orderTokenVaultInput.id}`
    );
    if (!takeOrderTokenVault) {
      takeOrderTokenVault = new TokenVaultTakeOrder(
        `${takeOrderEntity.id}-${orderTokenVaultInput.id}`
      );
      takeOrderTokenVault.wasInput = true;
      takeOrderTokenVault.wasOutput = false;
      takeOrderTokenVault.takeOrder = takeOrderEntity.id;
      takeOrderTokenVault.tokenVault = orderTokenVaultInput.id;
      takeOrderTokenVault.save();
    }
  }

  // Updating order input/output balance
  const orderTokenVaultOutput = TokenVault.load(tokenVaultOutput);
  if (orderTokenVaultOutput) {
    orderTokenVaultOutput.balance = orderTokenVaultOutput.balance.minus(
      event.params.input
    );
    orderTokenVaultOutput.balanceDisplay = toDisplay(
      orderTokenVaultOutput.balance,
      orderTokenVaultOutput.token
    );
    orderTokenVaultOutput.save();

    let takeOrderTokenVault = TokenVaultTakeOrder.load(
      `${takeOrderEntity.id}-${orderTokenVaultOutput.id}`
    );
    if (!takeOrderTokenVault) {
      takeOrderTokenVault = new TokenVaultTakeOrder(
        `${takeOrderEntity.id}-${orderTokenVaultOutput.id}`
      );
      takeOrderTokenVault.wasInput = false;
      takeOrderTokenVault.wasOutput = true;
      takeOrderTokenVault.takeOrder = takeOrderEntity.id;
      takeOrderTokenVault.tokenVault = orderTokenVaultOutput.id;
      takeOrderTokenVault.save();
    }
  }
}

export function handleWithdraw(event: Withdraw): void {
  let tokenVault = createTokenVault(
    event.params.vaultId.toString(),
    event.params.sender,
    event.params.token
  );

  if (tokenVault) {
    tokenVault.balance = tokenVault.balance.minus(event.params.amount);
    tokenVault.balanceDisplay = toDisplay(
      tokenVault.balance,
      event.params.token.toHexString()
    );
    tokenVault.save();
  }

  let vaultWithdraw = createVaultWithdraw(event.transaction.hash.toHex());
  vaultWithdraw.sender = createAccount(event.params.sender).id;
  vaultWithdraw.token = createToken(event.params.token).id;
  vaultWithdraw.vaultId = event.params.vaultId;
  vaultWithdraw.vault = createVault(
    event.params.vaultId.toString(),
    event.params.sender
  ).id;
  vaultWithdraw.requestedAmount = event.params.targetAmount;
  vaultWithdraw.requestedAmountDisplay = toDisplay(
    event.params.targetAmount,
    event.params.token.toHexString()
  );
  vaultWithdraw.amount = event.params.amount;
  vaultWithdraw.amountDisplay = toDisplay(
    vaultWithdraw.amount,
    event.params.token.toHexString()
  );
  vaultWithdraw.tokenVault = tokenVault.id;
  vaultWithdraw.vault = createVault(
    event.params.vaultId.toString(),
    event.params.sender
  ).id;
  vaultWithdraw.transaction = createTransaction(
    event.transaction.hash.toHex(),
    event.block
  ).id;
  vaultWithdraw.emitter = createAccount(event.params.sender).id;
  vaultWithdraw.timestamp = event.block.timestamp;
  vaultWithdraw.save();
}

export function handleMetaV1(event: MetaV1): void {
  const metaV1 = getRainMetaV1(event.params.meta);

  const subjectHex = getEvenHex(event.params.subject.toHex());

  // If the subject is equal to the emiter address, then the meta is for the OB.
  // In this scenario, it is considered that is from DeployerDiscoverableMeta.
  if (subjectHex == event.address.toHex()) {
    let orderBook = OrderBook.load(event.address);
    if (!orderBook) {
      orderBook = new OrderBook(event.address);
      orderBook.deployer = event.transaction.from;
      orderBook.address = event.address;
      orderBook.meta = metaV1.id;

      orderBook.save();
    }
  } else {
    // If not, the subject is an OrderHash then it's an Order meta
    const orderHash = getEvenHex(event.params.subject.toHex());
    const order = Order.load(orderHash);
    if (order) {
      order.meta = metaV1.id;
      order.save();
    }
  }

  // Converts the emitted target from Bytes to a Hexadecimal value
  let meta = event.params.meta.toHex();

  // Decode the meta only if incluse the RainMeta magic number.
  if (meta.includes(RAIN_META_DOCUMENT_HEX)) {
    meta = meta.replace(RAIN_META_DOCUMENT_HEX, "");
    const data = new CBORDecoder(stringToArrayBuffer(meta));
    const res = data.parse();

    const contentArr: ContentMeta[] = [];

    if (res.isSequence) {
      const dataString = res.toString();
      const jsonArr = json.fromString(dataString).toArray();
      for (let i = 0; i < jsonArr.length; i++) {
        const jsonValue = jsonArr[i];

        // if some value is not a JSON/Map, then is not following the RainMeta design.
        // So, return here to avoid assignation.
        if (jsonValue.kind != JSONValueKind.OBJECT) return;

        const jsonContent = jsonValue.toObject();

        // If some content is not valid, then skip it since is bad formed
        if (!ContentMeta.validate(jsonContent)) return;

        const content = new ContentMeta(jsonContent, metaV1.id);
        contentArr.push(content);
      }
    } else if (res.isObj) {
      const dataString = res.toString();
      const jsonObj = json.fromString(dataString).toObject();

      if (!ContentMeta.validate(jsonObj)) return;
      const content = new ContentMeta(jsonObj, metaV1.id);
      contentArr.push(content);
      //
    } else {
      // If the response is NOT a Sequence or an Object, then the meta have an
      // error or it's bad formed.
      // In this case, we skip to continue the decoding and assignation process.
      return;
    }

    for (let i = 0; i < contentArr.length; i++) {
      contentArr[i].generate();
    }
  } else {
    // The meta emitted does not include the RainMeta magic number, so does not
    // follow the RainMeta Desing
    return;
  }
}
export class ContentMeta {
  rainMetaId: Bytes;
  payload: Bytes = Bytes.empty();
  // eslint-disable-next-line @typescript-eslint/ban-types
  magicNumber: BigInt = BigInt.zero();
  contentType: string = "";
  contentEncoding: string = "";
  contentLanguage: string = "";

  constructor(
    metaContentV1Object_: TypedMap<string, JSONValue>,
    rainMetaID_: Bytes
  ) {
    const payload = metaContentV1Object_.get("0");
    const magicNumber = metaContentV1Object_.get("1");
    const contentType = metaContentV1Object_.get("2");
    const contentEncoding = metaContentV1Object_.get("3");
    const contentLanguage = metaContentV1Object_.get("4");

    // RainMetaV1 ID
    this.rainMetaId = rainMetaID_;

    // Mandatories keys
    if (payload) {
      let auxPayload = payload.toString();
      if (auxPayload.startsWith("h'")) {
        auxPayload = auxPayload.replace("h'", "");
      }
      if (auxPayload.endsWith("'")) {
        auxPayload = auxPayload.replace("'", "");
      }

      this.payload = Bytes.fromHexString(auxPayload);
    }

    // if (payload) this.payload = payload.toString();
    if (magicNumber) this.magicNumber = magicNumber.toBigInt();

    // Keys optionals
    if (contentType) this.contentType = contentType.toString();
    if (contentEncoding) this.contentEncoding = contentEncoding.toString();
    if (contentLanguage) this.contentLanguage = contentLanguage.toString();
  }

  /**
   * Validate that the keys exist on the map
   */
  static validate(metaContentV1Object: TypedMap<string, JSONValue>): boolean {
    const payload = metaContentV1Object.get("0");
    const magicNumber = metaContentV1Object.get("1");
    const contentType = metaContentV1Object.get("2");
    const contentEncoding = metaContentV1Object.get("3");
    const contentLanguage = metaContentV1Object.get("4");

    // Only payload and magicNumber are mandatory on RainMetaV1
    // See: https://github.com/rainprotocol/specs/blob/main/metadata-v1.md
    if (payload && magicNumber) {
      if (
        payload.kind == JSONValueKind.STRING ||
        magicNumber.kind == JSONValueKind.NUMBER
      ) {
        // Check if payload is a valid Bytes (hexa)
        let auxPayload = payload.toString();
        if (auxPayload.startsWith("h'")) {
          auxPayload = auxPayload.replace("h'", "");
        }
        if (auxPayload.endsWith("'")) {
          auxPayload = auxPayload.replace("'", "");
        }

        // If the payload is not a valid bytes value
        if (!isHexadecimalString(auxPayload)) {
          return false;
        }

        // Check the type of optionals keys
        if (contentType) {
          if (contentType.kind != JSONValueKind.STRING) {
            return false;
          }
        }
        if (contentEncoding) {
          if (contentEncoding.kind != JSONValueKind.STRING) {
            return false;
          }
        }
        if (contentLanguage) {
          if (contentLanguage.kind != JSONValueKind.STRING) {
            return false;
          }
        }

        return true;
      }
    }

    return false;
  }

  private getContentId(): Bytes {
    // Values as Bytes
    const payloadB = this.payload;
    const magicNumberB = Bytes.fromHexString(this.magicNumber.toHex());
    const contentTypeB = Bytes.fromUTF8(this.contentType);
    const contentEncodingB = Bytes.fromUTF8(this.contentEncoding);
    const contentLanguageB = Bytes.fromUTF8(this.contentLanguage);

    // payload +  magicNumber + contentType + contentEncoding + contentLanguage
    const contentId = getKeccak256FromBytes(
      payloadB
        .concat(magicNumberB)
        .concat(contentTypeB)
        .concat(contentEncodingB)
        .concat(contentLanguageB)
    );

    return contentId;
  }

  /**
   * Create or generate a MetaContentV1 entity based on the current fields:
   *
   * - If the MetaContentV1 does not exist, create the MetaContentV1 entity and
   * made the relation to the rainMetaId.
   *
   * - If the MetaContentV1 does exist, add the relation to the rainMetaId.
   */
  generate(): MetaContentV1 {
    const contentId = this.getContentId();

    let metaContent = MetaContentV1.load(contentId);

    if (!metaContent) {
      metaContent = new MetaContentV1(contentId);

      metaContent.payload = this.payload;
      metaContent.magicNumber = this.magicNumber;
      metaContent.documents = [];

      if (this.contentType != "") metaContent.contentType = this.contentType;

      if (this.contentEncoding != "")
        metaContent.contentEncoding = this.contentEncoding;

      if (this.contentLanguage != "")
        metaContent.contentLanguage = this.contentLanguage;
    }

    const aux = metaContent.documents;
    if (!aux.includes(this.rainMetaId)) aux.push(this.rainMetaId);

    metaContent.documents = aux;

    metaContent.save();

    return metaContent;
  }
}
