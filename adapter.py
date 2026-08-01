"""
Thin adapter bridging original textdistance tests to the Rust CLI binary.
Usage: pytest tests/original/ --tb=short
"""

import json
import subprocess
import sys
from pathlib import Path

BINARY = Path(__file__).parent / "target" / "release" / "textdistance-rs"
if not BINARY.exists():
    BINARY = Path(__file__).parent / "target" / "debug" / "textdistance-rs"


def _run(algorithm: str, *sequences: str, **kwargs) -> dict:
    """Call the Rust CLI and return parsed JSON output."""
    args = [str(BINARY), algorithm]
    for k, v in kwargs.items():
        kebab = k.replace("_", "-")
        if isinstance(v, bool):
            if v:
                args.append(f"--{kebab}")
        elif isinstance(v, list):
            for item in v:
                args.append(f"--{kebab}={item}")
        elif v is not None:
            args.append(f"--{kebab}={v}")
    args.extend(sequences)
    result = subprocess.run(args, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"CLI error: {result.stderr.strip()}")
    return json.loads(result.stdout)


# ── Algorithm wrapper classes ──

class _AlgorithmWrapper:
    """Wraps a Rust algorithm, exposing distance/similarity/normalized methods."""
    def __init__(self, name: str, default_kwargs: dict = None):
        self._name = name
        self._default_kwargs = default_kwargs or {}

    def __call__(self, *sequences: str) -> float:
        data = _run(self._name, *sequences, **self._default_kwargs)
        # Return primary measure: similarity for similarity algos, distance for distance algos
        # Heuristic: if 'similarity' key exists and distance != similarity...
        # Actually, all outputs have both. Return distance by default (matches Base.__call__)
        return data["distance"]

    def distance(self, *sequences: str) -> float:
        data = _run(self._name, *sequences, **self._default_kwargs)
        return data["distance"]

    def similarity(self, *sequences: str) -> float:
        data = _run(self._name, *sequences, **self._default_kwargs)
        return data["similarity"]

    def normalized_distance(self, *sequences: str) -> float:
        data = _run(self._name, *sequences, **self._default_kwargs)
        return data["normalized_distance"]

    def normalized_similarity(self, *sequences: str) -> float:
        data = _run(self._name, *sequences, **self._default_kwargs)
        return data["normalized_similarity"]

    def maximum(self, *sequences: str) -> float:
        data = _run(self._name, *sequences, **self._default_kwargs)
        return data["distance"] + data["similarity"]

    def __repr__(self):
        return f"{self._name}({self._default_kwargs})"


class Hamming(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("hamming")

class Levenshtein(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("levenshtein")

class DamerauLevenshtein(_AlgorithmWrapper):
    def __init__(self, restricted=True, external=True):
        super().__init__("damerau-levenshtein", {"restricted": restricted})

class JaroWinkler(_AlgorithmWrapper):
    def __init__(self, winklerize=True, external=True):
        super().__init__("jaro-winkler", {"winklerize": winklerize})

class Jaro(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("jaro")

class NeedlemanWunsch(_AlgorithmWrapper):
    def __init__(self, gap_cost=1.0, gap_extension_cost=0.5, external=True):
        super().__init__("needleman-wunsch")

    def normalized_distance(self, *sequences):
        d = self.distance(*sequences)
        # NeedlemanWunsch has special normalization
        return d

    def normalized_similarity(self, *sequences):
        s = self.similarity(*sequences)
        return s

class SmithWaterman(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("smith-waterman")

class Gotoh(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("gotoh")

class StrCmp95(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("strcmp95")

class MLIPNS(_AlgorithmWrapper):
    def __init__(self, threshold=0.5, external=True):
        super().__init__("mlipns")

class LCSSeq(_AlgorithmWrapper):
    def __init__(self, qval=1, test_func=None, external=True):
        super().__init__("lcsseq")

class LCSStr(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("lcsstr")

class RatcliffObershelp(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("ratcliff-obershelp")

class Jaccard(_AlgorithmWrapper):
    def __init__(self, qval=1, as_set=False, external=True):
        super().__init__("jaccard", {"qval": qval, "as_set": as_set})

class Sorensen(_AlgorithmWrapper):
    def __init__(self, qval=1, as_set=False, external=True):
        super().__init__("sorensen", {"qval": qval, "as_set": as_set})

class Tversky(_AlgorithmWrapper):
    def __init__(self, qval=1, ks=None, bias=None, as_set=False, external=True):
        kwargs = {"qval": qval, "as_set": as_set}
        if ks:
            kwargs["ks"] = ks
        if bias is not None:
            kwargs["bias"] = bias
        super().__init__("tversky", kwargs)

class Overlap(_AlgorithmWrapper):
    def __init__(self, qval=1, as_set=False, external=True):
        super().__init__("overlap", {"qval": qval, "as_set": as_set})

class Cosine(_AlgorithmWrapper):
    def __init__(self, qval=1, as_set=False, external=True):
        super().__init__("cosine", {"qval": qval, "as_set": as_set})

class Tanimoto(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("tanimoto")

class Bag(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("bag")

class MRA(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("mra")

class Editex(_AlgorithmWrapper):
    def __init__(self, local=False, match_cost=0, group_cost=1, mismatch_cost=2, external=True):
        super().__init__("editex")

class Prefix(_AlgorithmWrapper):
    def __init__(self, qval=1, sim_test=None):
        super().__init__("prefix")

class Postfix(_AlgorithmWrapper):
    def __init__(self, qval=1):
        super().__init__("postfix")

class Length(_AlgorithmWrapper):
    def __init__(self):
        super().__init__("length")

class Identity(_AlgorithmWrapper):
    def __init__(self):
        super().__init__("identity")

class Matrix(_AlgorithmWrapper):
    def __init__(self, mat=None, mismatch_cost=0, match_cost=1, symmetric=True, external=True):
        super().__init__("matrix", {"mismatch_cost": mismatch_cost, "match_cost": match_cost})


# ── Module-level algorithm instances (matching original API) ──

bag = Bag()
cosine = Cosine()
dice = Sorensen()
jaccard = Jaccard()
monge_elkan = None  # Deferred
overlap = Overlap()
sorensen = Sorensen()
sorensen_dice = Sorensen()
tanimoto = Tanimoto()
tversky = Tversky()

hamming = Hamming()
levenshtein = Levenshtein()
damerau_levenshtein = DamerauLevenshtein()
jaro = Jaro()
jaro_winkler = JaroWinkler()
mlipns = MLIPNS()
needleman_wunsch = NeedlemanWunsch()
smith_waterman = SmithWaterman()
gotoh = Gotoh()
strcmp95 = StrCmp95()

lcsseq = LCSSeq()
lcsstr = LCSStr()
ratcliff_obershelp = RatcliffObershelp()

mra = MRA()
editex = Editex()

prefix = Prefix()
postfix = Postfix()
length = Length()
identity = Identity()
matrix = Matrix()
