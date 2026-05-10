#!/usr/bin/env bash
set -euo pipefail
curl -s http://localhost:8080/auth/login \
  -H 'content-type: application/json' \
  -d '{"email":"admin@cedar-rapids.example","password":"demo-password"}' \
  | python3 -m json.tool
