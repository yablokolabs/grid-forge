#!/usr/bin/env bash
set -euo pipefail
TOKEN=${TOKEN:?Set TOKEN from /auth/login first}
curl -s http://localhost:8080/customer-interactions/classify \
  -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"rawText":"A tree is touching the line behind my house and buzzing when it rains."}' \
  | python3 -m json.tool
