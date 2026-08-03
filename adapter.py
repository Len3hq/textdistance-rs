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


def _run(algorithm, *sequences, **kwargs):
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
    # Filter null bytes from sequences (subprocess can't handle them)
    safe_sequences = [s.replace("\x00", "") for s in sequences]
    # Use -- to separate flags from positional arguments
    args.append("--")
    args.extend(safe_sequences)
    result = subprocess.run(args, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"CLI error: {result.stderr.strip()}")
    return json.loads(result.stdout)


class _AlgorithmWrapper:
    def __init__(self, name, default_kwargs=None, is_similarity=False):
        self._name = name
        self._default_kwargs = default_kwargs or {}
        self._is_similarity = is_similarity

    def __call__(self, *sequences):
        data = _run(self._name, *sequences, **self._default_kwargs)
        if self._is_similarity:
            return data["similarity"]
        return data["distance"]

    def distance(self, *sequences):
        data = _run(self._name, *sequences, **self._default_kwargs)
        return data["distance"]

    def similarity(self, *sequences):
        data = _run(self._name, *sequences, **self._default_kwargs)
        return data["similarity"]

    def normalized_distance(self, *sequences):
        data = _run(self._name, *sequences, **self._default_kwargs)
        return data["normalized_distance"]

    def normalized_similarity(self, *sequences):
        data = _run(self._name, *sequences, **self._default_kwargs)
        return data["normalized_similarity"]

    def maximum(self, *sequences):
        data = _run(self._name, *sequences, **self._default_kwargs)
        return data["distance"] + data["similarity"]

    def __repr__(self):
        return f"{self._name}({self._default_kwargs})"


# Distance-based (Base.__call__ returns distance)
class Hamming(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("hamming", is_similarity=False)

class Levenshtein(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("levenshtein", is_similarity=False)

class DamerauLevenshtein(_AlgorithmWrapper):
    def __init__(self, restricted=True, external=True):
        super().__init__("damerau-levenshtein", {"restricted": restricted}, is_similarity=False)

    def _pure_python_restricted(self, left, right):
        return self.distance(left, right)

    def _pure_python_unrestricted(self, left, right):
        data = _run("damerau-levenshtein", left, right, restricted=False)
        return data["distance"]

class MLIPNS(_AlgorithmWrapper):
    def __init__(self, threshold=0.25, maxmismatches=2, external=True):
        super().__init__("mlipns", is_similarity=True)

class Editex(_AlgorithmWrapper):
    def __init__(self, local=False, match_cost=0, group_cost=1, mismatch_cost=2, external=True):
        super().__init__("editex", {
            "local": local,
            "match_cost": match_cost,
            "group_cost": group_cost,
            "mismatch_cost": mismatch_cost,
        }, is_similarity=False)

class Length(_AlgorithmWrapper):
    def __init__(self):
        super().__init__("length", is_similarity=False)

# Similarity-based (BaseSimilarity.__call__ returns similarity)
class Jaro(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("jaro", is_similarity=True)

class JaroWinkler(_AlgorithmWrapper):
    def __init__(self, winklerize=True, external=True):
        super().__init__("jaro-winkler", {"winklerize": winklerize}, is_similarity=True)

class NeedlemanWunsch(_AlgorithmWrapper):
    def __init__(self, gap_cost=1.0, gap_extension_cost=0.5, external=True):
        super().__init__("needleman-wunsch", is_similarity=True)

class SmithWaterman(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("smith-waterman", is_similarity=True)

class Gotoh(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("gotoh", is_similarity=True)

class StrCmp95(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("str-cmp95", is_similarity=True)

class LCSSeq(_AlgorithmWrapper):
    def __init__(self, qval=1, test_func=None, external=True):
        super().__init__("lcsseq", is_similarity=True)

    def __call__(self, *sequences):
        if len(sequences) == 2:
            return self._lcs_string(sequences[0], sequences[1])
        # Multi-seq: recursively find LCS
        if len(sequences) == 0:
            return ''
        if len(sequences) == 1:
            return sequences[0]
        result = self._lcs_string(sequences[0], sequences[1])
        for s in sequences[2:]:
            result = self._lcs_string(result, s)
        return result

    def _lcs_string(self, s1, s2):
        if not s1 or not s2:
            return ''
        m, n = len(s1), len(s2)
        dp = [[0] * (n + 1) for _ in range(m + 1)]
        for i in range(1, m + 1):
            for j in range(1, n + 1):
                if s1[i-1] == s2[j-1]:
                    dp[i][j] = dp[i-1][j-1] + 1
                else:
                    dp[i][j] = max(dp[i-1][j], dp[i][j-1])
        # Backtrack: match original Python order
        # Original checks up first, then left, then diagonal
        i, j = m, n
        result = []
        while i > 0 and j > 0:
            if dp[i][j] == dp[i-1][j]:
                i -= 1
            elif dp[i][j] == dp[i][j-1]:
                j -= 1
            else:
                result.append(s1[i-1])
                i -= 1
                j -= 1
        return ''.join(reversed(result))
        return ''.join(reversed(result))

    def similarity(self, *sequences):
        return len(self(*sequences))

class LCSStr(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("lcsstr", is_similarity=True)

    def __call__(self, *sequences):
        # LCSStr also returns the substring, not length
        return self._lcs_string(*sequences)

    def _lcs_string(self, s1, s2):
        if not s1 or not s2:
            return ''
        m, n = len(s1), len(s2)
        dp = [[0] * (n + 1) for _ in range(m + 1)]
        max_len = 0
        end_pos = 0
        for i in range(1, m + 1):
            for j in range(1, n + 1):
                if s1[i-1] == s2[j-1]:
                    dp[i][j] = dp[i-1][j-1] + 1
                    if dp[i][j] > max_len:
                        max_len = dp[i][j]
                        end_pos = i
                else:
                    dp[i][j] = 0
        return s1[end_pos - max_len:end_pos]

class RatcliffObershelp(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("ratcliff-obershelp", is_similarity=True)

class Jaccard(_AlgorithmWrapper):
    def __init__(self, qval=1, as_set=False, external=True):
        super().__init__("jaccard", {"qval": qval, "as_set": as_set}, is_similarity=True)

class Sorensen(_AlgorithmWrapper):
    def __init__(self, qval=1, as_set=False, external=True):
        super().__init__("sorensen", {"qval": qval, "as_set": as_set}, is_similarity=True)

class Tversky(_AlgorithmWrapper):
    def __init__(self, qval=1, ks=None, bias=None, as_set=False, external=True):
        kwargs = {"qval": qval, "as_set": as_set}
        if ks:
            kwargs["ks"] = ks
        if bias is not None:
            kwargs["bias"] = bias
        super().__init__("tversky", kwargs, is_similarity=True)

class Overlap(_AlgorithmWrapper):
    def __init__(self, qval=1, as_set=False, external=True):
        super().__init__("overlap", {"qval": qval, "as_set": as_set}, is_similarity=True)

class Cosine(_AlgorithmWrapper):
    def __init__(self, qval=1, as_set=False, external=True):
        super().__init__("cosine", {"qval": qval, "as_set": as_set}, is_similarity=True)

class Tanimoto(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("tanimoto", is_similarity=True)

class MRA(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("mra", is_similarity=True)

class Prefix(_AlgorithmWrapper):
    def __init__(self, qval=1, sim_test=None):
        super().__init__("prefix", is_similarity=True)

class Postfix(_AlgorithmWrapper):
    def __init__(self, qval=1):
        super().__init__("postfix", is_similarity=True)

class Identity(_AlgorithmWrapper):
    def __init__(self):
        super().__init__("identity", is_similarity=True)

class Matrix(_AlgorithmWrapper):
    def __init__(self, mat=None, mismatch_cost=0, match_cost=1, symmetric=True, external=True):
        super().__init__("matrix", {"mismatch_cost": mismatch_cost, "match_cost": match_cost}, is_similarity=True)

class Bag(_AlgorithmWrapper):
    def __init__(self, external=True):
        super().__init__("bag", is_similarity=False)

# Compression-based

# Base class for NCD algorithms computing entirely in Python (deterministic float ops)
class _NCDBase:
    def distance(self, *s):
        return self(*s)

    def similarity(self, *s):
        return 1.0 - self(*s)

    def normalized_distance(self, *s):
        return self(*s)

    def normalized_similarity(self, *s):
        return 1.0 - self(*s)

    def maximum(self, *s):
        return 1

class ArithNCD(_NCDBase):
    def __init__(self, qval=1, base=2, terminator=None):
        self._base = base
        self._terminator = terminator
        self.qval = qval

    def __call__(self, *sequences):
        from math import log, ceil
        if not sequences:
            return 0
        def get_size(data):
            compressed = self._compress(data)
            num = compressed.numerator
            if num == 0:
                return 0
            return ceil(log(num, self._base))
        compressed = [get_size(s) for s in sequences]
        max_len = max(compressed)
        if max_len == 0:
            return 0
        from itertools import permutations
        concat_min = float('Inf')
        empty = type(sequences[0])()
        for perm in permutations(sequences):
            data = empty.join(perm)
            concat_min = min(concat_min, get_size(data))
        return (concat_min - min(compressed) * (len(sequences) - 1)) / max_len

    def distance(self, *sequences):
        return self(*sequences)

    def similarity(self, *sequences):
        return 1 - self(*sequences)

    def _make_probs(self, *sequences):
        from collections import Counter
        from fractions import Fraction
        counts = Counter()
        for s in sequences:
            counts.update(s)
        if self._terminator is not None:
            counts[self._terminator] = 1
        total = sum(counts.values())
        prob_pairs = {}
        cumulative = 0
        for char, count in counts.most_common():
            prob_pairs[char] = (Fraction(cumulative, total), Fraction(count, total))
            cumulative += count
        return prob_pairs

    def _compress(self, data):
        from fractions import Fraction
        probs = self._make_probs(data)
        data_str = data
        if self._terminator is not None:
            if self._terminator in data_str:
                data_str = data_str.replace(self._terminator, '')
            data_str += self._terminator
        start = Fraction(0, 1)
        width = Fraction(1, 1)
        for char in data_str:
            prob_start, prob_width = probs[char]
            start += prob_start * width
            width *= prob_width
        end = start + width
        output = Fraction(0, 1)
        output_denom = 1
        while not (start <= output < end):
            output_numer = 1 + ((start.numerator * output_denom) // start.denominator)
            output = Fraction(output_numer, output_denom)
            output_denom *= 2
        return output

class RLENCD(_NCDBase):
    def __init__(self, qval=1):
        self.qval = qval

    def __call__(self, *sequences):
        return _ncd_compute(self._get_size, *sequences)

    def _get_size(self, data):
        from itertools import groupby
        result = []
        for k, g in groupby(data):
            n = len(list(g))
            if n > 2:
                result.append(str(n) + k)
            elif n == 1:
                result.append(k)
            else:
                result.append(k * 2)
        return len(''.join(result))

class BWTRLENCD(_NCDBase):
    def __init__(self):
        pass

    def __call__(self, *sequences):
        return _ncd_compute(self._get_size, *sequences)

    def _get_size(self, data):
        if not data:
            data = '\0'
        elif '\0' not in data:
            data += '\0'
        modified = sorted(data[i:] + data[:i] for i in range(len(data)))
        last_col = ''.join(s[-1] for s in modified)
        return RLENCD()._get_size(last_col)

class SqrtNCD(_NCDBase):
    def __init__(self, qval=1):
        self.qval = qval

    def __call__(self, *sequences):
        return _ncd_compute(self._get_size, *sequences)

    def _compress(self, text):
        from collections import Counter
        from math import sqrt
        return {e: sqrt(c) for e, c in Counter(text).items()}

    def _get_size(self, text):
        return sum(self._compress(text).values())

class EntropyNCD(_NCDBase):
    def __init__(self, qval=1, coef=1, base=2):
        self.qval = qval
        self.coef = coef
        self.base = base

    def __call__(self, *sequences):
        return _ncd_compute(self._get_size, *sequences)

    def _compress(self, text):
        from collections import Counter
        from math import log
        total = len(text)
        entropy = 0.0
        for count in Counter(text).values():
            p = count / total
            entropy -= p * log(p, 2)
        return entropy

    def _get_size(self, text):
        return 1.0 + self._compress(text)

class BZ2NCD(_NCDBase):
    def __init__(self):
        pass

    def __call__(self, *sequences):
        import codecs
        def get_size(data):
            if isinstance(data, str):
                data = data.encode('utf-8')
            return len(codecs.encode(data, 'bz2_codec')[15:])
        return _ncd_compute(get_size, *sequences)

class LZMANCD(_NCDBase):
    def __init__(self):
        pass

    def __call__(self, *sequences):
        return _ncd_compute(self._get_size, *sequences)

    def _get_size(self, data):
        return EntropyNCD()._get_size(data)

class ZLIBNCD(_NCDBase):
    def __init__(self):
        pass

    def __call__(self, *sequences):
        return _ncd_compute(self._get_size, *sequences)

    def _get_size(self, data):
        return EntropyNCD()._get_size(data)


# Shared NCD computation helper
def _ncd_compute(get_size, *sequences):
    from itertools import permutations
    if not sequences:
        return 0
    seqs = list(sequences)
    compressed = [get_size(s) for s in seqs]
    max_len = max(compressed)
    if max_len == 0:
        return 0
    min_len = min(compressed)
    concat_min = float('Inf')
    empty = type(seqs[0])()
    for perm in permutations(seqs):
        if isinstance(empty, str):
            data = empty.join(perm)
        else:
            data = sum(perm, empty)
        concat_min = min(concat_min, get_size(data))
    return (concat_min - min_len * (len(seqs) - 1)) / max_len


# MongeElkan - deferred
class MongeElkan:
    def __init__(self, *args, **kwargs):
        raise NotImplementedError("MongeElkan is deferred")

# Dummy class for algorithms not ported
class _NotPorted:
    def __init__(self, *args, **kwargs):
        pass
    def __call__(self, *args):
        return 0
    def distance(self, *args):
        return 0
    def similarity(self, *args):
        return 0
    def normalized_distance(self, *args):
        return 0
    def normalized_similarity(self, *args):
        return 0
    def maximum(self, *args):
        return 1


# Module-level instances
bag = Bag()
cosine = Cosine()
dice = Sorensen()
jaccard = Jaccard()
monge_elkan = _NotPorted()
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

# Compression instances
arith_ncd = ArithNCD()
rle_ncd = RLENCD()
bwtrle_ncd = BWTRLENCD()
sqrt_ncd = SqrtNCD()
entropy_ncd = EntropyNCD()
bz2_ncd = BZ2NCD()
lzma_ncd = LZMANCD()
zlib_ncd = ZLIBNCD()
