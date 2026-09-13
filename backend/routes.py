from flask import Blueprint, jsonify, request

from .models import UserProfile, db, upsert_profile

api_blueprint = Blueprint("api", __name__)


@api_blueprint.get("/health")
def health_check():
    return jsonify({"status": "ok"})


@api_blueprint.post("/profiles")
def create_or_update_profile():
    data = request.get_json(silent=True) or {}

    try:
        profile, created = upsert_profile(data)
    except ValueError:
        return jsonify({"error": "invalid profile payload"}), 400

    return jsonify(profile.to_dict()), 201 if created else 200


@api_blueprint.get("/profiles/<handle>")
def get_profile(handle):
    profile = UserProfile.query.filter_by(handle=handle).one_or_none()
    if profile is None:
        return jsonify({"error": "profile not found"}), 404

    return jsonify(profile.to_dict())


@api_blueprint.get("/profiles/<handle>/matrix")
def get_profile_matrix(handle):
    profile = UserProfile.query.filter_by(handle=handle).one_or_none()
    if profile is None:
        return jsonify({"error": "profile not found"}), 404

    return jsonify(
        {
            "handle": profile.handle,
            "matrix_kin_signature": profile.matrix_kin_signature,
            "numerology_matrix": profile.numerology_matrix,
            "calendar_matrix": profile.calendar_matrix,
        }
    )


@api_blueprint.post("/profiles/<handle>/recalculate")
def recalculate_profile_matrix(handle):
    profile = UserProfile.query.filter_by(handle=handle).one_or_none()
    if profile is None:
        return jsonify({"error": "profile not found"}), 404

    profile.refresh_matrix()
    db.session.commit()

    return jsonify(profile.to_dict())
