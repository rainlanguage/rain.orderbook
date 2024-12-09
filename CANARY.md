# Canary strat
- what is a canary order, what it does and what is its purpose?
canary is a simple order that sets a non-zero amount with a 0 io ratio, this means that the strat just gives money to the bot unconditionally, there's a timer on the ensure so it doesn't insta-drain itself, this can be used to check that the bot clears, e.g. after making changes or bringing up a new bot etc. to make sure that the infra is working correctly.

- where to find the strat?
https://github.com/rainlanguage/rain.dex.pubstrats/blob/main/src/infra/canary.rain

- additional info
canary orders are like any other orders, but they clear once every `cooldown` period, so if they happen to not clear for any reason the bot's exec kpi alert will go off, notifying that infra is possibly not working correctly, besides these there is nothing special about canaries.
now that bot has self fudning owned vaults option, one can specify the canary vault in the bot's env vars and bot will redeposit into the specified vault once the it goes below the specified threshold, this is not specific to canaries, but any vault can be set to be refunded. for more info about this, check bot's [README](https://github.com/rainlanguage/arb-bot/blob/master/README.md), but can simply set self fund vaults by follwoing example:
```sh
SELF_FUND_ORDERS=token1,vaultId1,threshold,toptupamount;token2,vaultId2,threshold,toptupamount;...
```