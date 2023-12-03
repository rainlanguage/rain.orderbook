use ethers::prelude::*;

abigen!(
  RainterpreterExpressionDeployer,
  "tests/generated/RainterpreterExpressionDeployerNPE2.json",
  derives(serde::Deserialize, serde::Serialize);

  Rainterpreter,
  "tests/generated/RainterpreterNPE2.json";

  RainterpreterStore,
  "tests/generated/RainterpreterStoreNPE2.json";

  RainterpreterParser,
  "tests/generated/RainterpreterParserNPE2.json";

  Orderbook,
  "tests/generated/OrderBook.json";

  ERC20Test,
  "tests/generated/ERC20Test.json";
);
