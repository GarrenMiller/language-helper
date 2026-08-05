# HFSTOL Binary File Format

HFSTOL (HFST Optimized Lookup) is a compact binary transducer format designed
for fast string lookup—hundreds of thousands of words per second. It is used
primarily for morphological analysis. This document describes the binary
layout so you can write your own reader / lookup tool without depending on
the HFST C++ library.

File extension: `.hfstol`

## References

- [OptimizedLookupFormat (HFST wiki)](https://github.com/hfst/hfst/wiki/OptimizedLookupFormat)
- [HFST Runtime Format — A Compacted Transducer Format Allowing for Fast
  Lookup](http://www.ling.helsinki.fi/~klinden/pubs/fsmnlp2009runtime.pdf)
- C++ implementation:
  [`libhfst/src/implementations/optimized-lookup/`](https://github.com/hfst/hfst/tree/master/libhfst/src/implementations/optimized-lookup)

---

## 1. High-Level Layout

The file is a flat binary stream with four sequential sections:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Header  (variable; 56-byte fixed portion after properties)│
├─────────────────────────────────────────────────────────────┤
│ 2. Alphabet (null-terminated UTF-8)                         │  variable
├─────────────────────────────────────────────────────────────┤
│ 3. Transition Index Table  6 × index_table_entries bytes    │
├─────────────────────────────────────────────────────────────┤
│ 4. Transition Table        8 × transition_table_entries bytes│
└─────────────────────────────────────────────────────────────┘
```

All multi-byte integers are **little-endian** and written via raw `write()` of
C/C++ types.

---

## 2. Header

There are **two header variants** depending on the HFSTOL version that
produced the file. A reader should detect which variant is present.

### 2.1 Variant detection

Read the first 5 bytes of the file:

- If they are `48 46 53 54 00` (ASCII `HFST` plus a NUL): **property-based
  header** (§2.2).
- Otherwise: **flat fixed-size header** (§2.3).

The flat variant was produced by older versions of HFST. The property-based
variant is produced by HFST 3.x+ and the `hfst-ospell` library.

### 2.2 Property-based header (HFST 3.x+)

The file begins with a 5-byte `HFST\0` magic (the ASCII letters `HFST`
followed by a NUL terminator), followed by a variable-length **property
section** that stores metadata as key-value pairs. The 56-byte fixed header
(§2.4) comes **after** the property section ends.

#### Property encoding

After the magic, the property section is laid out as:

```
uint8  magic[5]    "HFST" plus a NUL terminator
uint16 body_len    total length of the property body, in bytes (little-endian)
uint8  separator   always NUL
...    body        body_len bytes
```

The body is a single flat blob of concatenated NUL-terminated strings. The
strings are stored as alternating name/value pairs: each property's name
string is immediately followed by its value string, and both are terminated
by NUL.

```
name1\0value1\0name2\0value2\0 ... nameN\0valueN\0
```

The NUL that terminates the last value string is also the final byte of the
body (the byte at `body[body_len - 1]` is always NUL). There is **no
per-property length field**, no count of properties, and no end-of-properties
sentinel; the fixed header begins immediately after the body.

Typical properties: `version`, `type`, `formulaic-definition`, `name`. The
`type` value indicates the transducer flavour: `HFST_OL` for unweighted
optimized-lookup transducers or `HFST_OLW` for weighted ones.

#### Example (from `dog.hfstol`)

```
48 46 53 54 00   "HFST\0" magic
68 00            body_len = 104 (little-endian)
00               separator
76 65 72 73 69 6f 6e 00        "version"
33 2e 33 00                    "3.3"
74 79 70 65 00                 "type"
48 46 53 54 5f 4f 4c 00        "HFST_OL"
66 6f 72 6d 75 6c 61 69 63 2d 64 65 66 69 6e 69 74 69 6f 6e 00  "formulaic-definition"
54 20 2f 00                    "T /"
6e 61 6d 65 00                 "name"
74 65 78 74 28 2f 68 6f 6d 65 2f 67 61 72 72 65 6e 2f 50 72 6f 6a 65 63 74 73 2f 65 6d 4d 6f 72 70 68 2f 68 66 73 74 2f 64 6f 67 2e 61 74 74 29 00  "text(/home/garren/Projects/emMorph/hfst/dog.att)"
<56-byte fixed header starts here>
```

Properties parsed from the body:

| name | value |
|------|-------|
| `version` | `3.3` |
| `type` | `HFST_OL` |
| `formulaic-definition` | `T /` |
| `name` | `text(/home/garren/Projects/emMorph/hfst/dog.att)` |

#### Parsing pseudocode

```
read 5 bytes magic           // must be "HFST\0"
body_len = read_u16le()
read 1 byte separator        // must be NUL
body     = read_bytes(body_len)
assert body[body_len - 1] == 0

// Split body on NUL bytes, taking the strings two at a time as
// (name, value) pairs.
for each pair (name, value) in split_on_nul(body):
    store(name, value)
// Fixed header follows at current position
```

### 2.3 Flat fixed-size header (legacy)

In the legacy format the file has **no magic bytes** and the 56-byte header
begins at offset 0. The fixed fields are identical to §2.4 below.

### 2.4 Fixed header fields

Whether reached directly (legacy) or after the property section (HFST 3.x+),
the fixed portion is the same flat 56-byte sequence of C types with no padding:

| Relative offset | Type             | Size | Field                                |
|-----------------|------------------|------|--------------------------------------|
| 0               | `uint16`         | 2    | `number_of_input_symbols`            |
| 2               | `uint16`         | 2    | `number_of_symbols` (total alphabet) |
| 4               | `uint32`         | 4    | `size_of_transition_index_table`     |
| 8               | `uint32`         | 4    | `size_of_transition_target_table`    |
| 12              | `uint32`         | 4    | `number_of_states` (informational)   |
| 16              | `uint32`         | 4    | `number_of_transitions` (info)       |
| 20              | `uint32` (0\|1)  | 4    | `weighted`                           |
| 24              | `uint32` (0\|1)  | 4    | `deterministic`                      |
| 28              | `uint32` (0\|1)  | 4    | `input_deterministic`                |
| 32              | `uint32` (0\|1)  | 4    | `minimized`                          |
| 36              | `uint32` (0\|1)  | 4    | `cyclic`                             |
| 40              | `uint32` (0\|1)  | 4    | `has_epsilon_epsilon_transitions`    |
| 44              | `uint32` (0\|1)  | 4    | `has_input_epsilon_transitions`      |
| 48              | `uint32` (0\|1)  | 4    | `has_input_epsilon_cycles`           |
| 52              | `uint32` (0\|1)  | 4    | `has_unweighted_input_epsilon_cycles`|

**Total: 56 bytes**

---

## 3. Alphabet

Immediately after the header, `number_of_symbols` null-terminated UTF-8
strings are concatenated.

Pseudo-code to read:

```
for i in 0..number_of_symbols:
    s = read_until_null_byte()
    symbol_table[i] = s
```

Symbol **index 0** is always the epsilon symbol: `@_EPSILON_SYMBOL_@`.

### Special symbol conventions

| Prefix pattern | Meaning              |
|----------------|----------------------|
| `@P.` … `@`   | Positive flag diacritic   |
| `@N.` … `@`   | Negative flag diacritic   |
| `@R.` … `@`   | Require flag diacritic    |
| `@D.` … `@`   | Disallow flag diacritic   |
| `@C.` … `@`   | Clear flag diacritic      |
| `@U.` … `@`   | Unify flag diacritic      |
| `@_UNKNOWN_SYMBOL_@`  | Unknown symbol               |
| `@_IDENTITY_SYMBOL_@` | Identity symbol (any→any)    |
| `@_DEFAULT_SYMBOL_@`  | Default fallback symbol      |

Any symbol matching `@I.` … `@` is treated as an "Insert" meta-arc (like
epsilon but carries a specific output).

---

## 4. Transition Index Table

An array of `size_of_transition_index_table` entries. Each entry is **6 bytes**.

For **unweighted** transducers:

| Offset | Type      | Size | Field                   |
|--------|-----------|------|-------------------------|
| 0      | `uint16`  | 2    | `input_symbol`          |
| 2      | `uint32`  | 4    | `first_transition_index`|

For **weighted** transducers the layout is identical, but the interpretation
of the final-weight field differs (see below).

### Sentinel values

| `input_symbol` | `first_transition_index` | Meaning         |
|----------------|--------------------------|-----------------|
| `0xFFFF`       | `1`                      | **Final index** — this state is accepting |
| `0xFFFF`       | `0xFFFFFFFF`             | **Empty** — no index here, skip           |
| anything else  | any valid index          | Normal entry: when input matches this symbol, jump to the transition table at `first_transition_index` |

In a **weighted** transducer the final weight is recovered by reinterpreting
the `first_transition_index` field as an IEEE 754 `float` via type-punning.

---

## 5. Transition Table

An array of `size_of_transition_target_table` entries.

### Unweighted (8 bytes per entry)

| Offset | Type      | Size | Field           |
|--------|-----------|------|-----------------|
| 0      | `uint16`  | 2    | `input_symbol`  |
| 2      | `uint16`  | 2    | `output_symbol` |
| 4      | `uint32`  | 4    | `target_index`  |

### Weighted (12 bytes per entry)

Same as unweighted plus a 4-byte `float` weight at offset 8.

### Sentinel values

| `input_symbol` | `output_symbol` | `target_index` | Meaning              |
|----------------|-----------------|----------------|----------------------|
| `0xFFFF`       | `0xFFFF`        | `1`            | **Final transition** — accepting state inside the transition table |
| `0xFFFF`       | `0xFFFF`        | `0xFFFFFFFF`   | **End-of-state marker** — sentinel padding for a state with no transitions inside the transition table |
| `0`            | any             | any            | Epsilon transition (consumes no input) |

For a valid transition, `input_symbol` is a real symbol number. The jump target
is `target_index`.

---

## 6. The Two-Table Indexing Scheme

This is the central idea of the format. States are addressed by their position
in the tables. Two kinds of states are distinguished by a **threshold
constant**:

```
TRANSITION_TARGET_TABLE_START = 0x80000000  (2³¹ = 2,147,483,648)
```

- **Index < 2³¹:** The state is in the **Transition Index Table** — a "full"
  state that uses indexed lookups for fast dispatch.

- **Index ≥ 2³¹:** The state is in the **Transition Table** directly — a
  "small" state (typically only one input symbol) that is traversed linearly.
  Subtract `TRANSITION_TARGET_TABLE_START` to get the actual index.

### Navigating a full state (index table)

Given state position `S` (a value < 2³¹):

1. Check `index_table[S + 1]` for epsilon transitions (`input_symbol == 0`).
   If present, follow the leading epsilon transitions starting at the target
   in the transition table.
2. For input symbol `N`, read `index_table[S + 1 + N]`. If
   `input_symbol == N`, follow transitions in the transition table at
   `first_transition_index`.
3. Check finality of `S` by checking `index_table[S]` for the final sentinel.

### Navigating a small state (transition table)

Given state position `T` (a value ≥ 2³¹), let `t = T - TRANSITION_TARGET_TABLE_START`:

1. The entry at `transition_table[t]` is a boundary sentinel
   (`input_symbol == 0xFFFF`).
2. Starting from `t + 1`, read transitions sequentially. Each transition's
   `input_symbol` is checked against the current input.
3. Stop when a sentinel (`input_symbol == 0xFFFF`) is reached.
4. The boundary sentinel may also encode finality.

---

## 7. Tokenization (Encoder)

Before lookup, the input string must be converted to a sequence of symbol
numbers. The transducer's alphabet contains all possible input symbols, and a
**character trie** (`OlLetterTrie`) is built from them.

The trie supports greedy longest-match tokenization:

1. **ASCII fast path:** For bytes 0–127, a flat 128-entry array maps each
   byte to a symbol number in O(1), provided that byte is a single-character
   input symbol.
2. **Non-ASCII / multi-character:** The trie is traversed character by
   character, picking the longest matching string.
3. **Unknown fallback:** If no symbol matches, the next UTF-8 codepoint is
   tokenized as-is and added to the alphabet on the fly (dynamic alphabet
   extension).

The result is a `SymbolNumber` sequence (the *input tape*), terminated by
`NO_SYMBOL_NUMBER` (`0xFFFF`).

---

## 8. Output Tape

During lookup the transducer builds an *output tape* — a sequence of
`(input_symbol, output_symbol)` pairs. When a final state is reached:

- Input symbols that correspond to consumed input characters appear on both
  sides (e.g., the characters of the word being analyzed).
- Epsilon transitions have `input_symbol == 0` and a non-zero `output_symbol`
  (e.g., morphological tags like `+NOUN`, `+SG`).
- Meta-arcs (identity, unknown, default) have their output symbol resolved
  from the corresponding position in the input tape.

The output tape is serialized as an analysis string by concatenating the
non-epsilon output symbols, with each symbol represented by its string from
the alphabet table.

---

## 9. Flag Diacritics

Flag diacritics are a mechanism for enforcing constraints across long-distance
dependencies (e.g., subject-verb agreement). They are detected by the `@X.`
prefix in the alphabet and stored as an `FdTable` mapping symbol numbers to
`FdOperation` structs.

During lookup, a flag state tracks which features have been set, and flag
diacritic transitions are only followed if their operation succeeds:

| Flag | Operation                                |
|------|------------------------------------------|
| `P`  | Positive set — requires the feature be unset, then sets it |
| `N`  | Negative set — requires the feature be unset, then sets to negative |
| `R`  | Require — requires the feature be set (positive or negative) |
| `D`  | Disallow — requires the feature be unset |
| `C`  | Clear — unconditionally unsets the feature |
| `U`  | Unify — if set, requires matching value; if unset, sets it |

Flag transitions have `input_symbol == 0` in the index table (they index at
position `S + 1` just like epsilons) and appear before regular transitions in
the transition table.

---

## 10. Lookup Algorithm

The core lookup is a recursive depth-first search:

```
get_analyses(input_pos, output_pos, state_index):
    if input exhausted AND state is final:
        emit output path

    try epsilon/flag transitions (consume no input)

    if input exhausted:
        return

    symbol = next input symbol

    if symbol is in the alphabet:
        follow matching transitions
    else:
        try identity symbol transitions
        try unknown symbol transitions

    if no transition found AND default symbol exists:
        try default symbol transitions
```

Each matching transition advances `input_pos` by 1 and writes an
`(input_symbol, output_symbol)` pair to the output tape, then recurses into
the target state.

### Key constraints

- Maximum recursion depth is 5000 (prevents stack overflow).
- Input epsilon cycles are detected via a set of `(index, flag_state)` pairs
  visited at each input position.
- Results are collected in a set of `(weight, StringPairVector)` paths.

---

## 11. Example

From the wiki: a transducer mapping `dog` to morphological analyses.

```
States in AT&T notation:
0   1   d   d
1   2   o   o
2   3   g   g
3   4   @0@ +NOUN
3   6   @0@ +VERB
4   5   @0@ +SG
5                       (final)
6   5   @0@ +PRES
6   7   g   +PAST
7   8   e   @0@
8   5   d   @0@
```

### Alphabet (with symbol numbers):
| # | Symbol  |
|---|---------|
| 0 | `@0@`   |
| 1 | `d`     |
| 2 | `o`     |
| 3 | `g`     |
| 4 | `e`     |
| 5 | `+NOUN` |
| 6 | `+SG`   |
| 7 | `+VERB` |
| 8 | `+PAST` |
| 9 | `+PRES` |

### Analyses for `dog`:
```
dog+NOUN+SG
dog+VERB+PRES
```

### Analyses for `dogged`:
```
dog+VERB+PAST
```

### Binary layout (hex dump, unweighted, legacy flat-header format):

```
Header (no magic — starts at offset 0):
0005 ......... 5 input symbols
000a ......... 10 symbols total
000c 0000 .... 12 index table entries
0013 0000 .... 19 transition table entries
0009 0000 .... 9 states
0000 0000 .... (no boolean flags set; 9× uint32 of flags follow)

Alphabet:
3040 0040 ............... "@0@"
0064 .................... "d"
006f .................... "o"
0067 .................... "g"
0065 .................... "e"
4e2b 554f 004e .......... "+NOUN"
532b 0047 ............... "+SG"
562b 5245 0042 .......... "+VERB"
502b 5341 0054 .......... "+PAST"
502b 4552 0053 .......... "+PRES"

Transition Index Table (12 entries × 6 bytes):
(ffff ffff ffff) ... EMPTY (padding for index 0 - epsilon)
(ffff ffff ffff) ... EMPTY (padding)
(0001 0000 8000) ... d:X transitions at TA index 0
(0000 0009 8000) ... @0@:X transitions at TA index 9
(ffff ffff ffff) ... EMPTY
...

Transition Table (19 entries × 8 bytes):
0001 0001 0001 8000 ... d:d → TA index 1
ffff ffff ffff ffff ... non-final sentinel
0002 0002 0003 8000 ... o:o → TA index 3
ffff ffff ffff ffff ... non-final sentinel
0003 0003 0005 8000 ... g:g → TA index 5
ffff ffff ffff ffff ... non-final sentinel
0000 0005 0010 8000 ... @0@:+NOUN → TA index 16
0000 0007 0002 0000 ... @0@:+VERB → TIA index 2
ffff ffff ffff ffff ... non-final sentinel
0000 0009 000f 8000 ... @0@:+PRES → TA index 15
0003 0008 000b 8000 ... g:+PAST → TA index 11
ffff ffff ffff ffff ... non-final sentinel
0004 0000 000d 8000 ... e:@0@ → TA index 13
ffff ffff ffff ffff ... non-final sentinel
0001 0000 000f 8000 ... d:@0@ → TA index 15
ffff ffff 0001 0000 ... final sentinel (state accepts)
ffff ffff ffff ffff ... non-final sentinel
0000 0006 000f 8000 ... @0@:+SG → TA index 15
```

Key observations:
- TIA entries with `(0x8000)` in the high half of the target index denote a
  transition-table target (≥ TRANSITION_TARGET_TABLE_START).
- TIA entries with `(0x0000)` in the high half denote an index-table target
  (the `+VERB` transition jumps to TIA index 2, i.e. state 2, which uses
  index-based dispatch).
- The final state has `first_transition_index == 1` and
  `input_symbol == 0xFFFF`.

---

## 12. Writing a Minimal Reader

To implement a lookup tool from scratch:

1. **Detect the header variant** — read the first 5 bytes. If they are
   `HFST\0`, skip the magic, body length, separator, and property body
   (§2.2) before reading the fixed header. Otherwise, read the fixed header
   from offset 0 (§2.3).

2. **Read the header** — validate that fields are consistent (e.g.,
   `input_symbols ≤ symbols`, table sizes are positive).

3. **Read the alphabet** — build `symbol_table[0..N]` of UTF-8 strings.
   Identify flag diacritics and special symbols (`@_UNKNOWN_...`, etc.).

3. **Read the two tables** — deserialize the index and transition arrays into
   your language's equivalent structs.

4. **Build the trie** — for each input symbol (index 0 through
   `number_of_input_symbols`), insert its string into a character trie that
   maps strings → symbol numbers. Optionally, build a flat ASCII lookup for
   single-character symbols.

5. **Implement the lookup** — the recursive algorithm described in §10.

For reference, the C++ implementation lives in:
- [`transducer.h`](https://github.com/hfst/hfst/blob/master/libhfst/src/implementations/optimized-lookup/transducer.h) — data structures
- [`transducer.cc`](https://github.com/hfst/hfst/blob/master/libhfst/src/implementations/optimized-lookup/transducer.cc) — tokenization and lookup logic
