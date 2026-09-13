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
  created_at = db.Column(db.DateTime, server_default=db.func.current_timestamp())

  def to_dict(self):
    return {
        "id": self.id,
        "handle": self.handle,
        "evm_address": self.evm_address,
        "icp_principal": self.icp_principal,
        "matrix_kin_signature": self.matrix_kin_signature,
    }


with app.app_context():
  db.create_all()
