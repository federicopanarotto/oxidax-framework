#!/bin/sh
set -e
BASE=http://localhost:3000

echo "health: $(curl -s $BASE/health)"
echo "swagger-ui: $(curl -s -o /dev/null -w '%{http_code}' $BASE/swagger-ui/)"

CREATED=$(curl -s -X POST $BASE/api/reports \
  -H 'content-type: application/json' \
  --data '{"name":"Report E2E","description":"test contro server surrealdb"}')
echo "create: $CREATED"

ID=$(echo "$CREATED" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
echo "id: $ID"

echo "get:    $(curl -s $BASE/api/reports/$ID)"
echo "list:   $(curl -s $BASE/api/reports)"
echo "update: $(curl -s -X PUT $BASE/api/reports/$ID \
  -H 'content-type: application/json' \
  --data '{"name":"Report E2E v2","description":"aggiornato"}')"
echo "delete: $(curl -s -o /dev/null -w '%{http_code}' -X DELETE $BASE/api/reports/$ID)"
echo "get dopo delete: $(curl -s -o /dev/null -w '%{http_code}' $BASE/api/reports/$ID)"

echo "openapi paths:"
curl -s $BASE/api-docs/openapi.json | tr ',' '\n' | grep -o '"/api/reports[^"]*"'
