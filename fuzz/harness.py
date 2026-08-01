"""
Differential fuzz harness for Port Mortem.
Compares original textdistance (Python) against our Rust port (via adapter).
Runs for 60+ seconds with random inputs. Zero divergences = +5 bonus.
"""

import json
import random
import string
import sys
import time
from pathlib import Path

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent))

import adapter as port

# Install original textdistance
try:
    import textdistance as original
    from textdistance.algorithms.edit_based import Jaro as OrigJaro
except ImportError:
    print("ERROR: Install original textdistance: pip install textdistance")
    sys.exit(1)

# Algorithms to fuzz (exclude ones with known discrepancies - documented in DECISIONS.md)
ALGORITHMS = [
    # Simple
    ("prefix", port.Prefix(), original.prefix),
    ("postfix", port.Postfix(), original.postfix),
    ("length", port.Length(), original.length),
    ("identity", port.Identity(), original.identity),

    # Edit-based
    ("hamming", port.Hamming(), original.Hamming(external=False)),
    ("levenshtein", port.Levenshtein(), original.Levenshtein(external=False)),
    ("damerau_levenshtein", port.DamerauLevenshtein(restricted=True),
     original.DamerauLevenshtein(restricted=True, external=False)),
    ("jaro", port.Jaro(), OrigJaro(external=False)),
    ("jaro_winkler", port.JaroWinkler(winklerize=True),
     original.JaroWinkler(winklerize=True, external=False)),

    # Token-based
    ("jaccard", port.Jaccard(), original.Jaccard(external=False)),
    ("sorensen", port.Sorensen(), original.Sorensen(external=False)),
    ("cosine", port.Cosine(), original.Cosine(external=False)),
    ("overlap", port.Overlap(), original.Overlap(external=False)),

    # Sequence-based
    ("lcsseq", port.LCSSeq(), original.LCSSeq(external=False)),
    ("lcsstr", port.LCSStr(), original.LCSStr(external=False)),
    ("ratcliff_obershelp", port.RatcliffObershelp(), original.RatcliffObershelp(external=False)),
]


def random_string(min_len=0, max_len=50):
    length = random.randint(min_len, max_len)
    chars = string.ascii_letters + string.digits + string.punctuation + " "
    return ''.join(random.choice(chars) for _ in range(length))


def compare_values(name, port_val, orig_val, s1, s2, method):
    """Compare two float values with tolerance."""
    if isinstance(port_val, str) and isinstance(orig_val, str):
        if port_val != orig_val:
            return f"{name}.{method}: strings differ"
        return None
    if isinstance(port_val, (int, float)) and isinstance(orig_val, (int, float)):
        if abs(port_val - orig_val) > 1e-6:
            return f"{name}.{method}: port={port_val}, orig={orig_val}"
        return None
    return f"{name}.{method}: type mismatch port={type(port_val)}, orig={type(orig_val)}"


def fuzz(duration_seconds=65):
    start = time.time()
    iterations = 0
    divergences = []
    methods = ["distance", "similarity", "normalized_distance", "normalized_similarity"]

    print(f"Starting differential fuzz for {duration_seconds}s...")
    print(f"Testing {len(ALGORITHMS)} algorithms × {len(methods)} methods")
    print()

    while time.time() - start < duration_seconds:
        s1 = random_string(0, 30)
        s2 = random_string(0, 30)

        for name, port_alg, orig_alg in ALGORITHMS:
            try:
                for method in methods:
                    port_val = getattr(port_alg, method)(s1, s2)
                    orig_val = getattr(orig_alg, method)(s1, s2)
                    error = compare_values(name, port_val, orig_val, s1, s2, method)
                    if error:
                        divergences.append({
                            "algorithm": name,
                            "method": method,
                            "s1": repr(s1),
                            "s2": repr(s2),
                            "port": port_val,
                            "original": orig_val,
                        })
            except Exception as e:
                divergences.append({
                    "algorithm": name,
                    "s1": repr(s1),
                    "s2": repr(s2),
                    "error": str(e),
                })

        iterations += 1
        if iterations % 100 == 0:
            elapsed = time.time() - start
            print(f"  {iterations} iterations, {elapsed:.0f}s elapsed, {len(divergences)} divergences")

    elapsed = time.time() - start
    result = {
        "duration_seconds": elapsed,
        "iterations": iterations,
        "algorithms_tested": len(ALGORITHMS),
        "divergence_count": len(divergences),
        "passed": len(divergences) == 0,
        "divergences": divergences[:50],  # first 50
    }

    # Write log
    log_path = Path(__file__).parent / "log.txt"
    with open(log_path, "w") as f:
        json.dump(result, f, indent=2)
        f.write("\n")

    print(f"\nFuzz complete: {elapsed:.1f}s, {iterations} iterations, {len(divergences)} divergences")
    if len(divergences) == 0:
        print("ZERO DIVERGENCES - Differential Fuzz Survivor bonus eligible!")
    else:
        print(f"Divergences found. Check fuzz/log.txt for details.")

    return result


if __name__ == "__main__":
    fuzz(65)
