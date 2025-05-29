#!/bin/bash
# Fix for the Keylime registrar pool_size issue
# This script should be run inside the registrar container

# Find the problematic db.py file
DB_FILE=$(find /usr/local/lib/python3.12/site-packages/ -name db.py | grep keylime)

if [ -z "$DB_FILE" ]; then
    echo "Could not find db.py file"
    exit 1
fi

echo "Found db.py at $DB_FILE"

# Create a backup
cp "$DB_FILE" "${DB_FILE}.bak"

# Replace the problematic line
sed -i 's/p_sz, m_ovfl = p_sz_m_ovfl.split(",")/p_sz, m_ovfl = ("20", "10") if "," not in p_sz_m_ovfl else p_sz_m_ovfl.split(",")/' "$DB_FILE"

echo "Fixed db.py file"

# Verify the fix was applied
grep -A 2 "p_sz_m_ovfl" "$DB_FILE"