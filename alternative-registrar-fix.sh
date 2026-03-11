#!/bin/bash
# Alternative approach to fix registrar startup issues

# Script to apply within registrar container if first approach doesn't work

# Create a simple fixed Python script to start the registrar
cat > /tmp/fixed_registrar.py << 'EOF'
#!/usr/bin/env python3

import os
import sys
from pkg_resources import load_entry_point

# Monkey patch the problematic function in keylime models
try:
    from keylime.models.base import db
    original_make_engine = db.make_engine
    
    def patched_make_engine(config_key):
        try:
            return original_make_engine(config_key)
        except ValueError as e:
            if "not enough values to unpack" in str(e):
                print("Patching pool_size and max_overflow configuration")
                # Set default values
                db.engine = db.create_engine(
                    db.get_c_database_url(config_key),
                    pool_size=20,
                    max_overflow=10
                )
                return db.engine
            raise
    
    # Apply the patch
    db.make_engine = patched_make_engine
    print("Successfully applied database connection patch")
except ImportError as e:
    print(f"Failed to import required module: {e}")
    sys.exit(1)

# Start the registrar
try:
    sys.exit(load_entry_point('keylime==7.12.1', 'console_scripts', 'keylime_registrar')())
except Exception as e:
    print(f"Error starting registrar: {e}")
    sys.exit(1)
EOF

chmod +x /tmp/fixed_registrar.py
exec python3 /tmp/fixed_registrar.py