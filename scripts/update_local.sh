#!/bin/sh

# putting your discogs token here is the dumbest thing you can do.
token=$(cat ~/.config/cogsy/token)
username="cartoon.raccoon"
domain="https://api.discogs.com"

wantlist="$domain/users/$username/wants"
collection="$domain/users/$username/collection/folders/0/releases?per_page=100"

headers="Authorization: Discogs token=$token"

echo "Pulling wantlist..."
echo ""
curl "$wantlist" -H "$headers" | jq > ../discogs_wantlist.json

echo ""

echo "Pulling collection..."
echo ""
curl "$collection" -H "$headers" | jq > ../discogs_collection.json

