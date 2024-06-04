# Reth EIP-7702 Sandbox

```bash
export DEV_FOLDER="<your dev folder>"
git clone git@github.com:otim-xyz/alloy.git $DEV_FOLDER/alloy
git clone git@github.com:otim-xyz/reth.git $DEV_FOLDER/reth
git clone git@github.com:otim-xyz/reth-7702-sandbox.git $DEV_FOLDER/reth-7702-sandbox
cd $DEV_FOLDER/alloy && git switch eip-7702
cd $DEV_FOLDER/reth && git switch eip-7702
cd $DEV_FOLDER/reth-7702-sandbox
cargo run
```
