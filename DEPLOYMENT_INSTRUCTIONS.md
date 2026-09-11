# 🚀 Mainnet Deployment Instructions

## How to Trigger Deployment

1. **Go to GitHub Actions**
   - Navigate to: https://github.com/valorjesson-commits/market-earth-amm/actions
   - Select: `🚀 Market Earth AMM Mainnet Deployment`

2. **Click "Run workflow"**
   - Branch: `main`
   - Fill in required inputs:
     - **Governance Principal**: Your multisig principal (format: `xxxxx-xxxxx-xxxxx-xxxxx-cai`)
     - **Network**: Select `ic` for mainnet or `staging` for testnet

3. **Watch Progress in Real-Time**
   - Each step shows `[X%]` completion
   - Phases:
     - **10-20%**: Environment & CLI setup
     - **25-30%**: Build WASM canisters
     - **35-40%**: Verification & checksums
     - **45-50%**: Pre-deployment checks
     - **55-70%**: Deploy to network
     - **75-85%**: Validate deployment
     - **90-100%**: Finalize & report

## Progress Breakdown

```
[10%] ████░░░░░░░░░░░░░░░░ Checkout
[15%] █████░░░░░░░░░░░░░░░ Rust setup
[20%] ██████░░░░░░░░░░░░░░ DFX install
[25%] ███████░░░░░░░░░░░░░ Build factory
[30%] ████████░░░░░░░░░░░░ Build pair
[35%] █████████░░░░░░░░░░░ Verify WASM
[40%] ██████████░░░░░░░░░░ Checksums
[45%] ███████████░░░░░░░░░ Validate principal
[50%] ████████████░░░░░░░░ DFX config
[55%] █████████████░░░░░░░ Setup identity
[60%] ██████████████░░░░░░ Deploy factory
[70%] ███████████████░░░░░ Verify factory
[75%] ████████████████░░░░ Get config
[80%] █████████████████░░░ List pairs
[85%] ██████████████████░░ Generate report
[90%] ███████████████████░ Upload artifacts
[95%] ████████████████████ Summary
[100%] █████████████████████ COMPLETE! 🎉
```

## Output Files

After deployment, check the workflow run for:
- ✅ `DEPLOYMENT_REPORT.md` - Complete deployment details
- ✅ `checksums.txt` - WASM binary checksums for verification
- ✅ `Deployment Summary` - Quick reference in workflow output

## Verification

Once deployment completes, verify your canister:

```bash
# Get your factory canister ID from workflow output
FACTORY_ID="<from-workflow-output>"

# Check status
dfx canister --network ic status $FACTORY_ID

# Get configuration
dfx canister --network ic call $FACTORY_ID get_config

# List pairs (empty initially)
dfx canister --network ic call $FACTORY_ID list_pairs
```

## Next Steps After Deployment

1. **Deploy token canisters** (if not already live)
2. **Create your first pair**:
   ```bash
   dfx canister --network ic call $FACTORY_ID create_pair "(principal \"TOKEN_A_ID\", principal \"TOKEN_B_ID\")"
   ```
3. **Add liquidity** to the pair
4. **Execute test swaps** to verify AMM functionality

## Troubleshooting

### Workflow fails at "Deploy AMM Factory"
- Check GOV_PRINCIPAL format (must be valid IC principal)
- Ensure you have enough cycles in your wallet (>2T recommended)
- Check network connectivity

### Principal format issues
- Valid format: `xxxxx-xxxxx-xxxxx-xxxxx-cai` (52 characters)
- Run: `dfx identity get-principal` to get your current principal

### Need to rollback?
- Keep the WASM checksums from deployment
- Can reinstall canister with different code if needed

---

**Status**: Ready for mainnet deployment 🚀
