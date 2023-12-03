use ethers::prelude::*;

abigen!(
  RainterpreterExpressionDeployer,
  "tests/generated/RainterpreterExpressionDeployerNP.json",
  derives(serde::Deserialize, serde::Serialize);

  Rainterpreter,
  "tests/generated/RainterpreterNP.json";

  RainterpreterStore,
  "tests/generated/RainterpreterStore.json";

  Orderbook,
  "tests/generated/OrderBook.json";

  AuthoringMetaGetter,
  "tests/generated/AuthoringMetaGetter.json";

  ERC20Mock,
  "tests/generated/ERC20Test.json";
);
