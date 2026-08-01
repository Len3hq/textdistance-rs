"""
pytest conftest — redirects 'import textdistance' to our adapter.
Test files in tests/original/ remain completely untouched.
"""

import sys
import adapter as textdistance

# Replace textdistance in sys.modules so all imports hit our adapter
sys.modules["textdistance"] = textdistance
sys.modules["textdistance.algorithms"] = textdistance
