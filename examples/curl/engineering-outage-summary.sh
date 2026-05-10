#!/usr/bin/env bash
set -euo pipefail
TOKEN=${TOKEN:?Set TOKEN from /auth/login first}
curl -s http://localhost:8080/engineering/outage-summary \
  -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"outageNumber":"OUT-2026-0417","fieldNotes":"Feeder F-12 locked out after storm. AMI last-gasp cluster near Oak Substation. Crew reports possible tree contact."}' \
  | python3 -m json.tool
