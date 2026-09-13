from flask import Blueprint, jsonify, request
from ledger import CrossChainLedger
from models import UserProfile, db

api_blueprint = Blueprint("api", __name__)


@api_blueprint.route("/swap", methods=["POST"])
def execute_swap_log():
  data = request.json
  record = CrossChainLedger.record_swap(
      user_id=data.get("user_id"),
      source_chain=data.get("source_chain"),
      target_chain=data.get("target_chain"),
      amount_in=data.get("amount_in"),
      amount_out=data.get("amount_out"),
  )
  return jsonify({"status": "success", "record": record}), 200
