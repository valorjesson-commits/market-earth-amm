from __future__ import annotations

import hashlib
import json
from datetime import date, datetime

from flask_sqlalchemy import SQLAlchemy

db = SQLAlchemy()

MASTER_NUMBERS = {11, 22, 33}


def _clean_text(value):
    if value is None:
        return None

    text = str(value).strip()
    return text or None


def _parse_date(value):
    if isinstance(value, date):
        return value

    value = _clean_text(value)
    if value is None:
        return None

    return datetime.fromisoformat(value).date()


def _reduce_number(value):
    if value <= 0:
        return 0

    while value > 9 and value not in MASTER_NUMBERS:
        value = sum(int(digit) for digit in str(value))

    return value


def _alpha_value(char):
    if not char.isalpha():
        return 0

    return ord(char.upper()) - 64


def _name_value(text):
    return _reduce_number(sum(_alpha_value(char) for char in text or ""))


def _digit_histogram(source):
    digits = "".join(char for char in source if char.isdigit())
    return {str(index): digits.count(str(index)) for index in range(1, 10)}


def build_profile_matrix(handle, full_name=None, birth_date=None, as_of=None):
    as_of = as_of or date.today()
    birth_date = birth_date or as_of

    display_name = full_name or handle
    birth_digits = birth_date.strftime("%Y%m%d")
    combined_digits = f"{birth_digits}{as_of.strftime('%Y%m%d')}"
    digit_histogram = _digit_histogram(combined_digits)

    life_path = _reduce_number(sum(int(digit) for digit in birth_digits))
    expression = _name_value(display_name)
    soul_urge = _name_value("".join(char for char in display_name if char.upper() in "AEIOU"))
    personality = _name_value("".join(char for char in display_name if char.upper() not in "AEIOU " and char.isalpha()))
    personal_year = _reduce_number(sum(int(digit) for digit in f"{as_of.year}{birth_date.month:02d}{birth_date.day:02d}"))
    personal_month = _reduce_number(personal_year + as_of.month)
    personal_day = _reduce_number(personal_month + as_of.day)

    calendar_matrix = {
        "as_of": as_of.isoformat(),
        "birth_date": birth_date.isoformat(),
        "personal_year": personal_year,
        "personal_month": personal_month,
        "personal_day": personal_day,
        "digit_histogram": digit_histogram,
    }

    numerology_matrix = {
        "life_path": life_path,
        "expression": expression,
        "soul_urge": soul_urge,
        "personality": personality,
    }

    signature_source = json.dumps(
        {
            "handle": handle,
            "full_name": full_name,
            "birth_date": birth_date.isoformat(),
            "calendar_matrix": calendar_matrix,
            "numerology_matrix": numerology_matrix,
        },
        sort_keys=True,
        separators=(",", ":"),
    )

    return {
        "matrix_kin_signature": hashlib.sha256(signature_source.encode("utf-8")).hexdigest(),
        "numerology_matrix": numerology_matrix,
        "calendar_matrix": calendar_matrix,
    }


class UserProfile(db.Model):
    __tablename__ = "user_profile"

    id = db.Column(db.Integer, primary_key=True)
    handle = db.Column(db.String(64), unique=True, nullable=False, index=True)
    full_name = db.Column(db.String(128), nullable=True)
    email = db.Column(db.String(255), unique=True, nullable=True)
    date_of_birth = db.Column(db.Date, nullable=True)
    city = db.Column(db.String(128), nullable=True)
    country = db.Column(db.String(128), nullable=True)
    icp_principal = db.Column(db.String(63), unique=True, nullable=True)
    wallet_address = db.Column(db.String(128), unique=True, nullable=True)
    matrix_kin_signature = db.Column(db.String(64), nullable=False, default="")
    numerology_matrix = db.Column(db.JSON, nullable=False, default=dict)
    calendar_matrix = db.Column(db.JSON, nullable=False, default=dict)
    created_at = db.Column(db.DateTime, server_default=db.func.current_timestamp(), nullable=False)
    updated_at = db.Column(
        db.DateTime,
        server_default=db.func.current_timestamp(),
        onupdate=db.func.current_timestamp(),
        nullable=False,
    )

    def refresh_matrix(self, as_of=None):
        matrix = build_profile_matrix(
            self.handle,
            full_name=self.full_name,
            birth_date=self.date_of_birth,
            as_of=as_of,
        )
        self.matrix_kin_signature = matrix["matrix_kin_signature"]
        self.numerology_matrix = matrix["numerology_matrix"]
        self.calendar_matrix = matrix["calendar_matrix"]
        return matrix

    def to_dict(self):
        return {
            "id": self.id,
            "handle": self.handle,
            "full_name": self.full_name,
            "email": self.email,
            "date_of_birth": self.date_of_birth.isoformat() if self.date_of_birth else None,
            "city": self.city,
            "country": self.country,
            "icp_principal": self.icp_principal,
            "wallet_address": self.wallet_address,
            "matrix_kin_signature": self.matrix_kin_signature,
            "numerology_matrix": self.numerology_matrix,
            "calendar_matrix": self.calendar_matrix,
            "created_at": self.created_at.isoformat() if self.created_at else None,
            "updated_at": self.updated_at.isoformat() if self.updated_at else None,
        }


def upsert_profile(data):
    handle = _clean_text(data.get("handle"))
    if not handle:
        raise ValueError("handle is required")

    profile = UserProfile.query.filter_by(handle=handle).one_or_none()
    created = profile is None
    if profile is None:
        profile = UserProfile(handle=handle)

    profile.full_name = _clean_text(data.get("full_name"))
    profile.email = _clean_text(data.get("email"))
    profile.date_of_birth = _parse_date(data.get("date_of_birth"))
    profile.city = _clean_text(data.get("city"))
    profile.country = _clean_text(data.get("country"))
    profile.icp_principal = _clean_text(data.get("icp_principal"))
    profile.wallet_address = _clean_text(data.get("wallet_address"))
    profile.refresh_matrix()

    db.session.add(profile)
    db.session.commit()
    return profile, created
