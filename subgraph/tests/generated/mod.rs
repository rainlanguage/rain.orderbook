use ethers::prelude::*;

abigen!(
  RainterpreterExpressionDeployer,
  "tests/generated/RainterpreterExpressionDeployerNP.json",
  derives(serde::Deserialize, serde::Serialize);

  Rainterpreter,
  "tests/generated/RainterpreterNP.json";

  RainterpreterStore,
  "tests/generated/RainterpreterStore.json";

  AuthoringMetaGetter,
  "tests/generated/AuthoringMetaGetter.json";

  OrderBook,
  "tests/generated/OrderBook.json";

  // ERC20Mock should not be replaced. It's for testing purpose
  ERC20Mock,
  "tests/generated/ERC20Mock.json";
);
