import { HttpAgent, Actor } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";
import { Principal } from "@dfinity/principal";

// Elements
const elProviderName = document.getElementById("wallet-provider-name");
const elPrincipalId = document.getElementById("wallet-principal-id");
const elBalance = document.getElementById("wallet-balance");
const elDisconnectBtn = document.getElementById("disconnect-btn");

const elIiConnectBtn = document.getElementById("ii-connect-btn");
const elNfidConnectBtn = document.getElementById("nfid-connect-btn");
const elPlugConnectBtn = document.getElementById("plug-connect-btn");
const elStoicConnectBtn = document.getElementById("stoic-connect-btn");

const elCanisterIdInput = document.getElementById("canister-id-input");
const elGetConfigBtn = document.getElementById("get-config-btn");
const elListPairsBtn = document.getElementById("list-pairs-btn");
const elCanisterLog = document.getElementById("canister-log-output");

// Global states
let authClient = null;
let currentIdentity = null;
let currentAgent = null;
let connectedProvider = null;

// Initialize AuthClient
async function initAuth() {
  try {
    authClient = await AuthClient.create();
    if (await authClient.isAuthenticated()) {
      handleAuthenticated("Internet Identity/NFID");
    }
  } catch (error) {
    console.error("Failed to initialize AuthClient:", error);
    logOutput(`Error initializing AuthClient: ${error.message}`);
  }
}

// Log function
function logOutput(message) {
  const timestamp = new Date().toLocaleTimeString();
  elCanisterLog.innerText = `[${timestamp}] ${message}\n` + elCanisterLog.innerText;
}

// Handle Successful Authentication
function handleAuthenticated(provider) {
  if (!authClient) return;
  
  currentIdentity = authClient.getIdentity();
  const principal = currentIdentity.getPrincipal();
  
  connectedProvider = provider;
  updateUIConnected(provider, principal.toString());
  
  // Set up agent
  const isLocal = window.location.hostname === "localhost" || window.location.hostname === "127.0.0.1";
  const host = isLocal ? "http://127.0.0.1:4943" : "https://ic0.app";
  
  currentAgent = new HttpAgent({ 
    identity: currentIdentity,
    host: host
  });

  if (isLocal) {
    currentAgent.fetchRootKey().catch(err => {
      console.warn("Failed to fetch root key for local agent:", err);
    });
  }
  
  logOutput(`Successfully connected using ${provider}!`);
}

// Update UI on Connection
function updateUIConnected(provider, principalId) {
  elProviderName.innerText = provider;
  elProviderName.className = "status-value";
  
  elPrincipalId.innerText = principalId;
  elPrincipalId.className = "status-value mono";
  
  elBalance.innerText = "0.00 ICP (Simulated)";
  
  // Toggle buttons
  elDisconnectBtn.removeAttribute("disabled");
  elGetConfigBtn.removeAttribute("disabled");
  elListPairsBtn.removeAttribute("disabled");
  
  elIiConnectBtn.setAttribute("disabled", "true");
  elNfidConnectBtn.setAttribute("disabled", "true");
  elPlugConnectBtn.setAttribute("disabled", "true");
  elStoicConnectBtn.setAttribute("disabled", "true");
}

// Update UI on Disconnect
function updateUIDisconnected() {
  elProviderName.innerText = "Not Connected";
  elProviderName.className = "status-value highlight";
  
  elPrincipalId.innerText = "-";
  elPrincipalId.className = "status-value mono";
  
  elBalance.innerText = "0.00 ICP";
  
  elDisconnectBtn.setAttribute("disabled", "true");
  elGetConfigBtn.setAttribute("disabled", "true");
  elListPairsBtn.setAttribute("disabled", "true");
  
  elIiConnectBtn.removeAttribute("disabled");
  elNfidConnectBtn.removeAttribute("disabled");
  elPlugConnectBtn.removeAttribute("disabled");
  elStoicConnectBtn.removeAttribute("disabled");
  
  connectedProvider = null;
  currentIdentity = null;
  currentAgent = null;
  
  logOutput("Disconnected wallet.");
}

// 1. Connect Internet Identity
async function connectInternetIdentity() {
  logOutput("Connecting with Internet Identity...");
  const isLocal = window.location.hostname === "localhost" || window.location.hostname === "127.0.0.1";
  
  const iiUrl = isLocal 
    ? `http://127.0.0.1:4943/?canisterId=rdg6j-jaaaa-aaaaa-aaada-cai` 
    : "https://identity.ic0.app";
    
  await authClient.login({
    identityProvider: iiUrl,
    onSuccess: () => {
      handleAuthenticated("Internet Identity");
    },
    onError: (err) => {
      logOutput(`Internet Identity login failed: ${err || 'unknown error'}`);
    }
  });
}

// 2. Connect NFID
async function connectNFID() {
  logOutput("Connecting with NFID...");
  const nfidUrl = "https://nfid.one/authenticate/?applicationName=Market+Earth+AMM";
  
  await authClient.login({
    identityProvider: nfidUrl,
    onSuccess: () => {
      handleAuthenticated("NFID");
    },
    onError: (err) => {
      logOutput(`NFID login failed: ${err || 'unknown error'}`);
    }
  });
}

// 3. Connect Plug Wallet
async function connectPlug() {
  logOutput("Connecting with Plug Wallet...");
  if (!window.ic || !window.ic.plug) {
    logOutput("Plug Wallet extension not detected. Please install it to connect.");
    window.open("https://plugwallet.ooo/", "_blank");
    return;
  }
  
  try {
    const isConnected = await window.ic.plug.requestConnect({
      whitelist: [], // list target canister IDs if known
      host: window.location.hostname === "localhost" ? "http://127.0.0.1:4943" : "https://ic0.app"
    });
    
    if (isConnected) {
      const principal = await window.ic.plug.getPrincipal();
      connectedProvider = "Plug Wallet";
      updateUIConnected("Plug Wallet", principal.toString());
      logOutput("Connected via Plug Wallet successfully!");
    } else {
      logOutput("Plug Connection rejected by user.");
    }
  } catch (error) {
    logOutput(`Plug Wallet connection error: ${error.message}`);
  }
}

// 4. Connect Stoic Wallet
async function connectStoic() {
  logOutput("Connecting with Stoic Wallet...");
  try {
    // Stoic wallet typically uses stoic-identity
    // In a production setup, we load StoicIdentity dynamically or import '@stoicwallet/identity'
    // Let's simulate/implement the Stoic login sequence:
    logOutput("Initializing Stoic login flow...");
    
    // For presentation/demo flow, we can prompt users with standard Stoic popup integration
    // We instantiate Stoic identity if the library is loaded or mock connection for UI
    const isMock = true; // since library might not be in our basic bundler dependencies, we support simulation or mock
    if (isMock) {
      setTimeout(() => {
        const mockPrincipal = "stoic-m7gny-x3vsz-aaaaa-cai";
        connectedProvider = "Stoic Wallet";
        updateUIConnected("Stoic Wallet", mockPrincipal);
        logOutput("Successfully authenticated with Stoic Wallet (Simulation)!");
      }, 800);
    }
  } catch (error) {
    logOutput(`Stoic connection error: ${error.message}`);
  }
}

// Disconnect Wallet
async function disconnect() {
  if (connectedProvider === "Internet Identity" || connectedProvider === "NFID") {
    await authClient.logout();
  }
  updateUIDisconnected();
}

// Call Canister methods
async function callCanisterMethod(methodName) {
  const canisterIdStr = elCanisterIdInput.value.trim();
  if (!canisterIdStr) {
    logOutput("Error: Please provide a Canister ID to interact with.");
    return;
  }
  
  try {
    const canisterId = Principal.fromText(canisterIdStr);
    logOutput(`Constructing Actor for Canister ${canisterIdStr}...`);
    
    // Define a minimal Candid interface for AMM Factory to call get_config / list_pairs
    const idlFactory = ({ IDL }) => {
      const Config = IDL.Record({
        'governance_principal' : IDL.Principal,
        'paused' : IDL.Bool,
        'whitelist' : IDL.Vec(IDL.Principal),
      });
      return IDL.Service({
        'get_config' : IDL.Func([], [Config], ['query']),
        'list_pairs' : IDL.Func([], [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Principal, IDL.Principal))], ['query']),
      });
    };

    const actor = Actor.createActor(idlFactory, {
      agent: currentAgent,
      canisterId: canisterId,
    });

    logOutput(`Calling ${methodName} on canister...`);
    let response;
    if (methodName === "get_config") {
      response = await actor.get_config();
      logOutput(`[Response] get_config(): ${JSON.stringify(response, (key, value) => {
        return typeof value === 'bigint' ? value.toString() : value;
      }, 2)}`);
    } else if (methodName === "list_pairs") {
      response = await actor.list_pairs();
      logOutput(`[Response] list_pairs(): ${JSON.stringify(response, null, 2)}`);
    }
  } catch (error) {
    logOutput(`Canister call failed: ${error.message}. (This is expected if the canister is not running locally, has a different ID, or the current user is not authorized.)`);
  }
}

// Tab Switching
document.querySelectorAll(".tab-btn").forEach(btn => {
  btn.addEventListener("click", () => {
    document.querySelectorAll(".tab-btn").forEach(b => b.classList.remove("active"));
    document.querySelectorAll(".tab-content").forEach(c => c.classList.remove("active"));
    
    btn.classList.add("active");
    document.getElementById(btn.dataset.tab).classList.add("active");
  });
});

// Event Listeners
elIiConnectBtn.addEventListener("click", connectInternetIdentity);
elNfidConnectBtn.addEventListener("click", connectNFID);
elPlugConnectBtn.addEventListener("click", connectPlug);
elStoicConnectBtn.addEventListener("click", connectStoic);
elDisconnectBtn.addEventListener("click", disconnect);

elGetConfigBtn.addEventListener("click", () => callCanisterMethod("get_config"));
elListPairsBtn.addEventListener("click", () => callCanisterMethod("list_pairs"));

// Initialize Auth
initAuth();
