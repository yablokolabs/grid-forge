#!/usr/bin/env bash
set -euo pipefail
TOKEN=${TOKEN:?Set TOKEN from /auth/login first}
curl -s http://localhost:8080/regulatory/draft \
  -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"docketNumber":"24-017","question":"Draft a response about quarterly reliability improvement reporting obligations."}' \
  | python3 -m json.tool
