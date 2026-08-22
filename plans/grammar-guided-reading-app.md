# Build Plan: Grammar-Guided Hungarian Reading App

Grammar-guided, low-latency language-learning app built around a small local
LLM. Inspired by the PDF in `~/Downloads/Optimizing Small LLMs for Language
Learning.pdf` (a Gemini conversation covering grammar-guided decoding, direct
token-ID manipulation, KV-cache optimization, and a "Contextual Reader /
X-Ray" app concept).

## Decisions

- **Form factor:** Web app (browser reader UI served by the existing axum
  backend).
- **Inference:** Pure-Rust runtime via **candle** (`candle-core` +
  `candle-transformers`). No GBNF support in candle, so we write our own
  **custom logit mask sampler** (the core learning project).
- **Model:** `Qwen2.5-3B-Instruct` (multilingual; decent Hungarian). GGUF
  Q4_K_S first; fallback to safetensors + candle int4 quantization if GGUF
  support is flaky.
- **Input sources (v1):** Paste/type into a textarea only. URL fetch, file
  upload, and browser extension are future work.
- **Analysis mode:** Eager with background prefill. Chunks are prefilled into
  the KV-cache in the background as the user scrolls so hover is fast.
- **HFST code:** Left dormant in the repo (unreferenced), recoverable via git
  history. Not part of the build.
- **Languages:** Hungarian first, architected so other languages can plug in
  later.

## Stack

- Rust 2024, axum (existing server)
- candle-core + candle-transformers
- `tokenizers` crate (same tokenizer as the model)
- serde / serde_json, anyhow
- hf-hub (or vendored GGUF)
- Static frontend served by axum

## Module Layout

```
src/
  ingest.rs        normalize -> tokenize+offsets -> chunk -> states
  slm/
    model.rs       load, forward pass, chat template
    kv_cache.rs    prefill, reuse, rollback snapshots
    trie.rs        byte-prefix trie over vocab
    sampler.rs     schema state machine + logit mask + sampling
    schema.rs      output structs (word, sentence, cloze, align)
  engine.rs        text_ready / interactive_ready orchestration + prefill worker
  handlers/        axum endpoints (keep morphology/vowel_harmony dormant)
frontend/          static reader UI
```

Repo changes: `crate::hfstol`, `hu.hfstol`, and the vowel-harmony handler are
left dormant (unreferenced, still in git history). The Dockerfile references a
stale `dog.hfstol` that must be cleaned up regardless.

## The Input Pipeline

### Where input comes from

| Source | How | Priority |
|---|---|---|
| Paste / type into textarea | Simplest; foundation for everything else | **v1** |
| URL fetch | POST a URL -> server fetches -> main-content extraction | v1.5 |
| File upload | `.txt`/`.md` first, `.epub` later | v2 |
| Browser extension | Select text on any page -> send to API | future |

### When is the input "ready" — two distinct moments

1. **`text_ready` (renderable)** — milliseconds after submission:
   - Server normalizes (NFKC, whitespace, sentence segmentation).
   - Tokenizes with the same `tokenizers` crate the model uses, recording a
     byte-offset map (every token knows its start/end bytes in the source).
   - Splits into context-sized chunks at sentence boundaries (with overlap).
   - Responds to the UI: `{ text, chunks[], token->offset map }`.
   - UI renders immediately. Hover/tap positions map to exact tokens with zero
     model involvement, because offsets come from the deterministic tokenizer,
     not the LLM.

2. **`interactive_ready` (hover is fast)** — after the KV-cache prefill:
   - Engine prefills system prompt + the sentence(s) around the hovered word
     into the KV-cache.
   - Static prefix stays cached; a word tap only appends ~a dozen query tokens
     and runs one forward pass -> masked sample of ~10-15 output tokens.
   - Eager mode: prefill the current chunk in the background as you scroll.
     UI shows an "analyzing..." indicator until `interactive_ready`.

Both states are explicit so the UI shows the right loading indicator.

### Chunking detail

The context window (e.g., 2048 tokens) is smaller than a full chapter. Chunks
split on sentence boundaries with a small overlap. Hovering a word uses the
chunk containing it. KV-cache rollback to the prompt boundary handles
switching words/chunks without re-running the static prefix.

## Highlight correctness without HFST

The model outputs values plus `token_index` ranges. The engine cross-checks
them against the tokenizer's byte-offset map before returning to the UI. If a
range is invalid, drop and re-run (the mask makes this rare; the tokenizer is
the source of truth for pixels).

## Phases

### Phase 1 — SLM Harness (prove the model runs)

1. Add deps. Write a CLI: load `Qwen2.5-3B-Instruct`, run one forward pass,
   greedy sample, print text.
2. Wire `tokenizers` (same tokenizer as model); implement the chat template.
3. Implement a minimal KV-cache struct (single forward with incremental eval).
4. Benchmark: TTFT, tok/s, model-load time, memory. Decide if CPU-only
   suffices or `candle-metal` is needed.
   - **Gate:** an unconstrained 15-token completion returns in the hundreds
     of ms.

### Phase 2 — Custom Logit Mask (the core learning project)

1. `trie.rs`: iterate the vocab, convert each token -> bytes, build a
   byte-prefix trie mapping byte-prefix -> candidate token ids.
2. `schema.rs`: define `WordBreakdown`
   (token_index, surface, lemma, pos, morph_tags, contextual_meaning,
   grammar_themes) and `SentenceAnalysis` (full_translation,
   aligned_segments[{token_range, target_text, themes}],
   grammar_themes_present).
3. `sampler.rs`: a `LogitMask` trait plus a JSON state machine per schema
   (`ExpectKey -> ExpectStartQuote -> ExpectValue -> ExpectEndQuote ->
   ExpectComma`). For each state: allowed byte patterns intersect trie
   candidates -> mask others to `-inf` -> sample (greedy first; add
   top-k/p/temperature).
   - **Gate:** a constrained `WordBreakdown` for a sample sentence parses via
     `serde_json::from_slice` 100% of the time.

### Phase 3 — Input Pipeline (paste-only) + "Ready" states

1. `ingest.rs`:
   - `POST /api/text {text}` -> normalize (NFKC, whitespace, sentence
     segmentation).
   - Tokenize; build the per-token byte-offset map.
   - Split into context-sized chunks at sentence boundaries.
   - Respond `{chunks[], offsets, state: "text_ready"}`. UI renders
     immediately.
2. Eager prefill worker: after `text_ready`, prefill system prompt + current
   chunk into the KV-cache in the background; emit `state:
   "interactive_ready"` when the hover path can answer in sub-50ms. Prefetch
   the next chunk on scroll (rollback to prompt boundary when switching).

### Phase 4 — Engine Endpoints

1. `POST /api/word {chunk_id, token_index}` -> masked `WordBreakdown`;
   cross-check token_range against the tokenizer offset map; drop/re-run on
   mismatch.
2. `POST /api/sentence {chunk_id, sentence_index}` -> `SentenceAnalysis`.
3. `POST /api/cloze {chunk_id, token_index}` -> fill-in-blank card (answer +
   distractors constrained by the mask).
4. Concurrency: single model instance behind a mutex/actor; analysis requests
   queue behind the prefill worker.

### Phase 5 — Web UI

1. Reader pane: text renders from `text_ready`; hover/tap -> overlay panel
   with morphology + contextual meaning + theme badges.
2. Side-by-side translation pane driven by `SentenceAnalysis` (hover syncs
   segments).
3. Click-to-filter by grammar theme (highlights all matching segments).
4. "analyzing..." indicator until `interactive_ready`.

## Functionality Brainstorm

Ideas that exploit the hybrid deterministic-tokenizer + masked-SLM
architecture. Ordered by priority.

1. **X-Ray Reader** (flagship): hover/tap any word -> contextual meaning +
   grammar-theme badge. Byte-exact highlighting, no fuzzy matching.
2. **Grammar-theme map of a whole chapter**: scan text, tag every segment
   with a predefined taxonomy enum, filter/highlight by theme (e.g., all
   SUBJUNCTIVE_MOOD).
3. **Side-by-side translation with alignment**: hover a clause on either side
   and the matching translation lights up; per-segment grammar themes.
4. **Adaptive cloze generator**: fill-in-the-blank cards whose answer +
   distractors are constrained by the mask; feed it the SRS review queue.
5. **"Choose Your Own Path" stories**: generated branching narratives where
   choices are constrained to `[VERB_ACTION] x [OBJECT_IN_SCENE]`, targeting
   missed grammar.
6. **Real-time grammar guardrails editor**: as you type, catch case/agreement
   errors at the exact token index and offer corrections, constrained to the
   current study level.
7. **Reading-level grader**: score text by vocabulary/grammar complexity;
   suggest simpler rewrites or flag "this chapter is above your level."
8. **IPA assistant with IPA-only mask**: context-aware transcription (liaison,
   vowel harmony) with zero symbol hallucination.
9. **Vowel-harmony conjugation trainer** (Hungarian-specific): deterministic
   suffix selection (definite/indefinite x front/back/rounded) with
   explanations of why.
10. **Morpheme-building game**: quiz users on suffix order and meaning from
    the model's morphological breakdown.
11. **Transfer-error diagnostics**: mask against a taxonomy of English ->
    Hungarian transfer errors and diagnose mistakes precisely.
12. **Derivation explorer**: cluster words sharing a derivational morpheme for
    vocabulary study.

## Open Items

- **GGUF quant support:** verify Q4_K_S loads in candle; have the safetensors
  path ready.
- **Hungarian quality of Qwen2.5-3B:** acceptable for v1; if lemma/POS is
  unreliable, the mask can restrict lemma output to a small vocab later.
- **Exact token-range correctness:** tokenizer offsets are ground truth;
  model ranges are validated against them (drop-and-rerun fallback).
