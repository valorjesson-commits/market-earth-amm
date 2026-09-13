from flask import Flask
from flask_sqlalchemy import SQLAlchemy

app = Flask(__name__)
app.config["SQLALCHEMY_DATABASE_URI"] = "sqlite:///market_earth.db"
db = SQLAlchemy(app)


class UserProfile(db.Model):
  __tablename__ = "user_profile"

  id = db.Column(db.Integer, primary_key=True)
  handle = db.Column(db.String(64), unique=True, nullable=False)
  evm_address = db.Column(db.String(42), unique=True, nullable=True)
  icp_principal = db.Column(db.String(63), unique=True, nullable=True)
  matrix_kin_signature = db.Column(db.String(32), nullable=True)
  net_worth = db.Column(db.Float, default=0.0, nullable=False)
  active_chains = db.Column(db.String(256), default="", nullable=False)
  is_verified = db.Column(db.Boolean, default=False, nullable=False)
  evm_balance = db.Column(db.Float, default=0.0, nullable=False)
  icp_balance = db.Column(db.Float, default=0.0, nullable=False)
  created_at = db.Column(db.DateTime, server_default=db.func.current_timestamp())

  def to_dict(self):
    return {
        "id": self.id,
        "handle": self.handle,
        "evm_address": self.evm_address,
        "icp_principal": self.icp_principal,
        "matrix_kin_signature": self.matrix_kin_signature,
        "net_worth": self.net_worth,
        "active_chains": self.active_chains,
        "is_verified": self.is_verified,
        "evm_balance": self.evm_balance,
        "icp_balance": self.icp_balance,
        "created_at": self.created_at.isoformat() if self.created_at else None,
    }

  def recalculate_net_worth(self):
    """
    Computes combined net worth (sum of EVM and ICP balances)
    and updates active chains list.
    """
    # Simulated native currency price conversion:
    # Let's say 1 ETH = $3000 USD, 1 ICP = $10 USD
    evm_usd = self.evm_balance * 3000.0 if self.evm_address else 0.0
    icp_usd = self.icp_balance * 10.0 if self.icp_principal else 0.0
    
    self.net_worth = evm_usd + icp_usd
    
    chains = []
    if self.evm_address and self.evm_balance > 0:
      chains.append("EVM")
    if self.icp_principal and self.icp_balance > 0:
      chains.append("ICP")
    
    self.active_chains = ",".join(chains)
    self.is_verified = len(chains) > 0

  def poll_and_update_balances(self, mock_evm_rpc=None, mock_ic_agent=None):
    """
    Simulates querying EVM RPCs and IC Agents to update user holdings and net worth.
    """
    if self.evm_address:
      if mock_evm_rpc:
        self.evm_balance = mock_evm_rpc(self.evm_address)
      else:
        # Simulate non-zero balance for a valid EVM address (e.g. 1.25 ETH)
        self.evm_balance = 1.25
    else:
      self.evm_balance = 0.0

    if self.icp_principal:
      if mock_ic_agent:
        self.icp_balance = mock_ic_agent(self.icp_principal)
      else:
        # Simulate non-zero balance for a valid ICP Principal (e.g. 75 ICP)
        self.icp_balance = 75.0
    else:
      self.icp_balance = 0.0

    self.recalculate_net_worth()


def sync_portfolio_holdings(user_id):
  """
  Service function to fetch, verify, and update a user's multi-chain balances.
  Queries mock EVM and ICP providers to pull live on-chain balances,
  recalculates combined net worth, active chains, and updates database.
  """
  user = db.session.get(UserProfile, user_id)
  if not user:
    return False
  
  user.poll_and_update_balances()
  db.session.commit()
  return True


with app.app_context():
  db.create_all()
