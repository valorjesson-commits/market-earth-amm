# Market Earth AMM - Wallet Connection Guide

This guide covers how to connect, manage, and interact with the Market Earth AMM canisters using both CLI (for developers) and Frontend (for DApp integration).

---

## 1. Developer CLI Wallet Connection (For Deployment & Testing)

Managing your developer identity and wallet is handled using the `dfx` CLI tool.

### Check Current Identity
Verify your current active CLI identity and its corresponding principal ID:
```bash
# Get active identity name (default is usually 'default')
dfx identity whoami

# Get current identity's principal ID
dfx identity get-principal
```

### Switch Active Identity
If you are managing multiple wallets or roles (such as a separate deployer or admin identity), switch the active identity:
```bash
dfx identity use <identity-name>
```

### Import an Existing Wallet/Identity
To import an existing mainnet identity from a PEM seed file or private key:
```bash
dfx identity import <identity-name> <path-to-pem-file>
```

### Associate & Verify Cycles Wallet
Ensure your active identity has sufficient cycles associated with it:
```bash
# Check local cycles balance
dfx wallet balance

# Check mainnet (ic) cycles wallet balance
dfx wallet --network ic balance
```

> **Tip**: You can use the pre-built CLI utility script `./scripts/wallet_manager.sh` included in this repository to automate these CLI steps!
> ```bash
> ./scripts/wallet_manager.sh status
> ./scripts/wallet_manager.sh use deployer
> ```

---

## 2. User Frontend Wallet Connection (For DApp Integration)

To allow users to connect their consumer wallets and perform actions on the Market Earth AMM, the frontend implements three primary integrations:

### A. Internet Identity & NFID
DFINITY's native Identity Anchors and NFID are integrated using `@dfinity/auth-client`.

```javascript
import { AuthClient } from "@dfinity/auth-client";
import { HttpAgent, Actor } from "@dfinity/agent";

// 1. Create the auth client
const authClient = await AuthClient.create();

// 2. Launch the login flow
const identityProvider = window.location.hostname === "localhost" 
  ? `http://127.0.0.1:4943/?canisterId=rdg6j-jaaaa-aaaaa-aaada-cai` 
  : "https://identity.ic0.app"; // OR "https://nfid.one/authenticate"

await authClient.login({
  identityProvider,
  onSuccess: () => {
    // 3. Get identity and build agent/actor
    const identity = authClient.getIdentity();
    const agent = new HttpAgent({ identity });
    
    // Connect to AMM Factory
    const factoryActor = Actor.createActor(idlFactory, {
      agent,
      canisterId: FACTORY_CANISTER_ID,
    });
  }
});
```

### B. Plug Wallet
Plug Wallet is a browser extension wallet connected using `window.ic.plug`.

```javascript
// 1. Request connection
const isConnected = await window.ic.plug.requestConnect({
  whitelist: [FACTORY_CANISTER_ID],
  host: "https://ic0.app"
});

if (isConnected) {
  // 2. Get principal ID
  const principal = await window.ic.plug.getPrincipal();
  console.log("Connected principal:", principal.toString());
}
```

### C. ICPSwap Wallet
ICPSwap wallet flows can use the same injected-wallet pattern as Plug-compatible Internet Computer wallets. The frontend now exposes a dedicated ICPSwap connect button that prefers `window.ic.icpswap` when available and falls back to the Plug-compatible API.

### D. Stoic Wallet
Stoic Wallet is another popular ICP wallet that can be connected via web flow using `@stoicwallet/identity`.

---

## 3. Pre-built Interactive Frontend

We have included a complete, responsive frontend application inside the `frontend` folder of this repository.

### Running the Frontend Locally

1. Install dependencies and start development server:
   ```bash
   cd frontend
   npm install
   npm run dev
   ```
2. Build static assets for canister deployment:
   ```bash
   npm run build
   ```

3. Deploy as an asset canister via `dfx`:
   ```bash
   dfx deploy frontend
   ```
