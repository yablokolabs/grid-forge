#!/usr/bin/env bash
set -euo pipefail
TOKEN=${TOKEN:?Set TOKEN from /auth/login first}
curl -s http://localhost:8080/search \
  -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"query":"vegetation outage restoration safety","module":"engineering","limit":3,"filters":{}}' \
  | python3 -m json.tool
