# Market Earth AMM

Automated Market Maker (AMM) deployment for Internet Computer with complete CI/CD pipeline.

## Quick Start

### Local Development
```bash
# Deploy locally
GOV_PRINCIPAL="$(dfx identity get-principal)" ./deploy_local.sh

# Run E2E tests
./scripts/integration_liquidity_test.sh

# Check mainnet status
./check_mainnet.sh
```

## Architecture

- **amm_factory**: Main factory canister for pair creation and management
- **pair**: Liquidity pool canister handling swaps and liquidity provision
- **CI/CD Pipeline**: Automated testing on push and pull requests

## Mainnet Deployment

See [docs/production_deploy.md](docs/production_deploy.md) for production deployment checklist.
