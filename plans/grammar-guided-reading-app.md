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
- **Model (v1 default):** `Qwen2.5-3B-Instruct` (multilingual; decent
  Hungarian).
- **Loading path (v1): GGUF quantized via `quantized_qwen2::ModelWeights`** —
  primary path on the Mac (CPU-friendly, ~2-2.5 GB for Q8_0 / Q4_K_M).
  Safetensors + `qwen2::ModelForCausalLM` is the alternative (needs Metal,
  BF16 ~6.4 GB). **First de-risk step:** confirm a Qwen2.5 GGUF loads in
  candle (tested path is Qwen2); fall back to `Qwen/Qwen2-1.5B-Instruct-GGUF`
  if not.
- **Model-agnostic design:** The mask core (trie, sampler, schema, engine) is
  model-agnostic and only talks to the model through a thin `Lm` trait plus
  tokenizer/prompt adapters. Swapping models later is one new adapter, not an
  engine change. Qwen2.5-3B is the default because it is the quality floor for
  Hungarian plus structured output — the mask constrains *format*, not
  semantic correctness.
- **Input sources (v1):** Paste/type into a textarea only. URL fetch, file
  upload, and browser extension are future work.
- **Analysis mode:** Eager with background prefill. The engine prefills the
  *fixed prefix* (system prompt + instruction + the current sentence) into the
  KV-cache as the user reads, so hovering a word in that sentence only appends
  the target word's tokens before generating. Because candle's KV-cache is
  opaque (no rollback), each new sentence means `clear` + re-prefill — so the
  prefix stays short.
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
    lm.rs          trait Lm { forward(..) -> logits } + tokenizer/prompt adapters
    backends/
      qwen2.rs     first Lm impl: Qwen2.5-3B-Instruct (GGUF, quantized_qwen2)
      (later) llama.rs
    kv_cache.rs    prefill pattern + clear/re-prefill (candle cache is opaque;
                   no snapshot/rollback — "rollback" = clear + re-prefill)
    trie.rs        byte-prefix trie over vocab
    sampler.rs     schema state machine + logit mask + sampling
    schema.rs      output structs (word, sentence, cloze, align)
  engine.rs        text_ready / interactive_ready orchestration + prefill worker
  handlers/        axum endpoints (keep morphology/vowel_harmony dormant)
frontend/          static reader UI
```

## Model-Agnostic Design

The mask engine must never know the model's name. Split the codebase into a
model-agnostic core and thin per-model adapters:

```
core (model-agnostic):   trie.rs, sampler.rs, schema.rs, engine.rs
traits:                  slm/lm.rs
adapter (one per arch):  slm/backends/qwen2.rs   (+ per-model chat template)
```

What the mask core depends on (all runtime parameters, never hardcoded):

- `trie.rs` — built at load time from whatever tokenizer the model uses.
  Operates on raw bytes, so it works across tokenizer styles (GPT `Ġ`,
  SentencePiece `▁`, multi-byte UTF-8).
- `sampler.rs` — the schema state machine and mask-to-`-inf` logic only touch
  the logits tensor.
- `engine.rs` — orchestration and ready-state logic.

What is genuinely model-specific (isolated behind the trait):

| Piece | Why | Where |
|---|---|---|
| Forward pass + KV-cache struct | Different structs/signatures per architecture in candle-transformers | `Lm` trait impl |
| Chat template | Qwen and Llama format prompts differently | prompt adapter |
| GGUF tensor names | candle maps GGUF weights per-architecture | backend loading path |
| Token ids (EOS/BOS) | Model-specific | adapter exposes them |

### The `Lm` contract (settled)

The trait defines the full set of operations the engine and sampler need from
any model. The engine builds prompts through the trait, so it never sees
model-specific markup or token ids.

```rust
pub trait Lm: Send {
    /// Run one forward pass over `tokens` starting at absolute position
    /// `seqlen_offset`; appends to the internal KV-cache; returns logits for
    /// the LAST token only, shape [1, vocab_size].
    fn forward(&mut self, tokens: &[u32], seqlen_offset: usize) -> Result<Tensor>;

    /// Empty the KV-cache (start a new conversation / rollback point).
    fn clear_kv_cache(&mut self);

    /// Token ids that stop generation.
    fn eos_tokens(&self) -> &[u32];

    /// The tokenizer: encode prompts, decode output, and in Phase 2 build the
    /// byte-prefix trie over the vocab.
    fn tokenizer(&self) -> &Tokenizer;

    /// Wrap system + user text into this model's chat format.
    fn format_chat(&self, system: &str, user: &str) -> String;
}
```

Method rationale:

- **`forward`** — the only method the mask engine calls
  (`lm.forward(tokens, offset) -> logits -> mask -> sample`). Takes `&[u32]`
  so the adapter owns device/tensor construction. Both candle backends return
  last-token logits, so the contract is uniform.
- **`clear_kv_cache`** — the engine's only cache control. candle's KV-cache is
  internal and opaque; there is no "rollback to position N". Hover flow per
  word is therefore: `clear -> forward(system, 0) -> forward(instruction +
  sentence, len) -> forward(target word) -> generate`. Re-prefill is the real
  cost, which is why prefixes stay short.
- **`eos_tokens`** — generation needs model-specific stop ids (Qwen:
  `<|endoftext|>` and `<|im_end|>`).
- **`tokenizer`** — the deterministic bridge: encode prompts, decode output,
  and (Phase 2) build the byte-prefix trie over the vocab.
- **`format_chat`** — keeps the engine free of model-specific markup
  (Qwen2.5's template is hardcoded, not jinja).

Deliberately left out of the trait: `device()` and `vocab_size()` — the sampler
can read vocab size from `logits.dims1()` at runtime and can hold the device
itself when rebuilding tensors after masking.

One subtlety for the mask core: subword tokens mean the state machine must
handle "mid-word" states — a general property that holds for any model.

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
   - Engine prefills the **fixed prefix** into the KV-cache: `system prompt +
     instruction + the sentence containing the hovered word`. The target word
     is NOT part of the prefix — hovering appends just its tokens.
   - Eager mode: prefill the visible sentence in the background as you read.
     UI shows an "analyzing..." indicator until `interactive_ready`.
   - Moving to a new sentence: `clear` + re-prefill its fixed prefix.

Both states are explicit so the UI shows the right loading indicator.

### Ingest is not "feeding the model"

`ingest.rs` never calls the model. It is pure, deterministic text processing:

```
paste text -> normalize (NFKC, whitespace, sentence split)
           -> tokenize with the tokenizers crate + build byte-offset map
           -> group tokens into sentences
```

It produces *structure* (token ids, byte offsets, sentence groups) that the UI
renders from and that makes hover-to-token mapping exact. The model only ever
sees a small, purpose-built prompt assembled per query:

```
system      ("You are a Hungarian language tutor...")   -- fixed
+ instruction ("Analyze the word at index N...")        -- fixed per endpoint
+ the sentence containing the word                       -- changes per sentence
+ the target word                                        -- changes per hover
```

The pasted text is never fed to the model wholesale; only the relevant
sentence + word go into a query prompt.

### Chunking detail

The context window (e.g., 2048 tokens) is smaller than a full chapter, so
text is grouped into sentences; larger documents split into chunks at sentence
boundaries with a small overlap. The **prefix that gets prefilled is short**:
system + instruction + the current sentence only — not a whole chunk. There is
no KV-cache rollback in candle, so switching sentences means `clear` +
re-prefill (accepting ~tens of ms per new sentence). Phase 4 may avoid
re-prefill entirely by generating a whole sentence's breakdown in one masked
pass, making hover a pure in-memory lookup.

## System Flow (post-Phase 1)

How the whole pipeline runs, from first input to last output:

```
 ①  INPUT (browser)
      user pastes target-language text
      │
      ▼  POST /api/text
 ┌────────────────────────────────────────────────────────────────────┐
 │ ingest.rs                                                           │
 │   • normalize (NFKC, whitespace, sentence split)                    │
 │   • tokenize + build byte-offset map   ◄── tokenizers crate         │
 │   • group tokens into sentences                                     │
 └────────────────────────────────────────────────────────────────────┘
      │  ②  { sentences, token→byte offsets }      → "text_ready"
      │     UI renders text immediately; hover positions map to tokens
      ▼
 ┌────────────────────────────────────────────────────────────────────┐
 │ engine.rs — eager prefill worker (background)                       │
 │   • model.clear_kv_cache()                                          │
 │   • lm.forward(system_prompt, 0)                                    │
 │   • lm.forward(instruction + sentence, len(system))                 │
 │   →  KV-cache holds the fixed prefix for this sentence              │
 │   → state: "interactive_ready"                                      │
 └────────────────────────────────────────────────────────────────────┘
      │  ③  user taps/hovers a word → POST /api/word {sentence, token_index}
      ▼
 ┌────────────────────────────────────────────────────────────────────┐
 │ slm/ — masked generation loop (Phase 2 machinery)                   │
 │   lm.forward(&[target_word_tokens], pos)      ◄── append the word     │
 │                                                                     │
 │   loop (per output token):                                          │
 │     logits  = lm.forward(&[tok], pos)        ◄── forward pass        │
 │     mask    = sampler.apply(trie, schema_state, logits)             │
 │     tok     = sample(mask)                                          │
 │     append bytes → advance schema state (JSON state machine)        │
 │   until schema complete → decode bytes → serde_json::from_slice     │
 └────────────────────────────────────────────────────────────────────┘
      │  ④  JSON: { token_index, lemma, pos, morph_tags, meaning, themes }
      ▼
 engine: validate token_index against tokenizer offset map
         (drop/re-run on mismatch)
      │  ⑤
      ▼  HTTP response
 ┌────────────────────────────────────────────────────────────────────┐
 │ ⑥  LAST OUTPUT — UI overlay: definition, POS/tag badges,           │
 │     contextual meaning, grammar-theme chips                         │
 └────────────────────────────────────────────────────────────────────┘
```

Key facts the diagram makes explicit:

- **Two readiness gates:** `② text_ready` (UI renders, no model involved) and
  the prefill step (hover becomes fast).
- **Hover skips re-prefill** within a sentence — it relies on the warm
  KV-cache. A new sentence = the worker re-prefills (clear + two short
  forwards).
- **The mask sits between forward and sample** — one loop iteration is exactly
  the Phase 2 core.
- **Last output is post-validated** against the tokenizer offset map before it
  reaches the UI.

## Highlight correctness without HFST

The model outputs values plus `token_index` ranges. The engine cross-checks
them against the tokenizer's byte-offset map before returning to the UI. If a
range is invalid, drop and re-run (the mask makes this rare; the tokenizer is
the source of truth for pixels).

## Phases

### Phase 0 — Toy trie + state machine exercise (no model, no candle)

Build the entire constrained-decoding mechanism in isolation — ~100 lines,
instant feedback, no model, no math — before touching the real 150k-token
vocabulary. Fully model-independent, so it can also serve as an early
orientation exercise. The modules built here are the *real* modules
(`slm/trie.rs`, `slm/sampler.rs`); later steps only swap the toy vocab for the
tokenizer's vocab and the toy grammar for the real schemas.

**0.1 Toy vocabulary** (test fixture, ~15 tokens):
```
enum values:  cat, car, dog, door, open, closed
structural:   {  }  "  :  ,  (space)
```

**0.2 `trie.rs` — byte-prefix trie** (generic over any vocab, reused later):
- `Trie::new(vocab: &[Vec<u8>]) -> Trie` — inserts each token's bytes, records
  token ids at terminal nodes.
- `valid_next_bytes(&self, prefix: &[u8]) -> Option<Vec<u8>>` — autocomplete:
  which next bytes continue any token.
- `tokens_with_prefix(&self, prefix: &[u8]) -> Vec<usize>` — which token ids
  have this prefix (this feeds the mask).
- Tests: insert `cat`/`car` -> `valid_next_bytes(b"c")` = `{a}`;
  `tokens_with_prefix(b"ca")` = `{cat, car}`; unknown prefix -> `None`.

**0.3 `sampler.rs` — toy JSON state machine**
Micro-grammar: output must be exactly `{"animal": "cat"}` or
`{"animal": "dog"}`.

```rust
enum ToyState {
  Start, KeyStart, Key(usize), KeyEnd,   // fixed key "animal", byte by byte
  Colon, Space, ValueStart,              // structural
  Value,                                  // enum: cat | dog  (the interesting state)
  ValueEnd, Close, Done,
}
```
- `advance(state, byte) -> Result<ToyState>` — errors on illegal bytes.
- `mask(state, partial_value_bytes, trie, vocab) -> Vec<usize>` — the heart of
  it:
  - `Value` state -> legal ids = enum ids (`cat`, `dog`) **intersected with**
    `trie.tokens_with_prefix(partial)` (prefix-filtered, exactly like phone
    autocomplete).
  - structural states -> the single-byte token matching the expected byte.
- Test: for every state, `mask` allows exactly the legal set.

**0.4 Simulated generation loop — the gate:**
- Loop: `legal = mask(state, …)` -> pick a legal token (iterate all choices
  across runs) -> append its bytes -> `state = advance(…)` -> repeat until
  `Done`.
- **Property test:** for *every* legal choice sequence, the produced string is
  always one of the two valid outputs — 100% of the time. Exhaustive over the
  enum values; repeat 1000x for determinism confidence.
- Optional: `src/bin/toy_generate.rs` prints a simulated generation so you can
  *see* it work.

**Not doing here:** no candle, no model, no logits, no softmax/temperature —
legality is just 0/1.

**Bridge to the real system:** in the real Phase 2, `mask` returns token ids
and the "cross-out" becomes `logits[id] = -inf` before sampling; the state
machine is hand-written per real schema (`WordBreakdown`, `SentenceAnalysis`,
`Cloze`) following the same pattern. Subword tokens (mid-word states) are the
one new complexity the toy intentionally skips.

### Phase 1 — Model-Agnostic Backbone

Build the seam that keeps the whole engine model-agnostic, not a throwaway
"does it run" spike. The code here is the foundation the later phases use
directly.

**What Phase 1 produces (three artifacts):**
1. `slm/lm.rs` — the `Lm` trait (the model-agnostic seam; contract above).
2. `slm/backends/qwen2.rs` — the Qwen2.5 adapter implementing the trait.
3. A working CLI + benchmark — proves load -> forward -> sample -> decode and
   produces the latency numbers that calibrate Phases 3-4.

**API facts (candle-transformers 0.11.0):**

Two supported loading paths; **GGUF quantized is the primary** on the Mac:

- Quantized / GGUF:
  ```rust
  use candle::quantized::gguf_file;
  use candle_transformers::models::quantized_qwen2::ModelWeights as Qwen2;
  let mut file = std::fs::File::open(&model_path)?;
  let content = gguf_file::Content::read(&mut file)?;
  let mut model = Qwen2::from_gguf(content, &mut file, &device)?;
  // model.forward(&tokens_tensor, index_pos)? -> last-token logits
  // model.clear_kv_cache()
  ```
  Supported quant types: `F32, F16, BF16, Q4_0, Q4_1, Q5_0, Q5_1, Q8_0, Q8_1,
  Q2K, Q3K, Q4K, Q5K, Q6K, Q8K`. Q4_K_M and Q8_0 are confirmed working.
  Start Q8_0 for quality; drop to Q4_K_M if RAM is tight.

- Safetensors / full precision (alternative):
  ```rust
  let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device)? };
  let config: qwen2::Config = serde_json::from_slice(&fs::read("config.json")?)?;
  let model = qwen2::ModelForCausalLM::new(&config, vb)?;
  // model.forward(&input_ids, seqlen_offset)? -> last-token logits
  ```
  `Qwen/Qwen2.5-3B-Instruct` is **sharded** (2 files + `model.safetensors.index.json`);
  BF16 weights ~6.4 GB (needs Metal). F32 on CPU is ~12 GB+.

**Shared semantics:**
- Both forwards take a `seqlen_offset` (absolute position of the input's first
  token) and return logits for the last token only — never slice logits
  yourself.
- Generation loop: first forward = full prompt tensor at offset `0`; each
  decode step = single-token tensor at `seqlen_offset = tokens.len() - 1`.
- Stop on both `<|endoftext|>` and `<|im_end|>` (Qwen).
- Tokenizer: `tokenizers::Tokenizer::from_file("tokenizer.json")`, encode with
  `add_special_tokens=true` (tokenizer adds BOS; do not prepend manually).
- Chat template is hardcoded (not jinja):
  ```
  <|im_start|>user\n{prompt}<|im_end|>\n<|im_start|>assistant\n
  ```
- Download via `hf-hub` sync API: `Api::new()?` -> `repo.get("tokenizer.json")`,
  `repo.get("config.json")`, and the weight file(s). Cache locally.
- Sampling: `candle_transformers::generation::LogitsProcessor::from_sampling(
  seed, Sampling::ArgMax)` for greedy now; temperature/top-k/top-p come free
  later via the same struct. (Phase 2's custom mask sits before this.)
- **Gotcha:** candle's `candle-examples` helper crate is not published on
  crates.io — reimplement `TokenOutputStream` (incremental decode) and the
  shard-index loader (~20 lines). `from_mmaped_safetensors` is `unsafe`.

**Deliverables checklist:**

| # | Deliverable | Notes |
|---|---|---|
| D1 | Cargo deps | `candle-core 0.11`, `candle-transformers 0.11`, `candle-nn 0.11`, `tokenizers 0.22`, `hf-hub 0.5` (tokio), `anyhow`. Optional `metal` feature on candle. |
| D2 | Model acquisition | Download GGUF + `tokenizer.json` (+ `config.json`) into a local cache dir; tiny loader module. |
| D3 | `slm/lm.rs` trait | The `Lm` contract (see Model-Agnostic Design). |
| D4 | `slm/backends/qwen2.rs` | One struct per path (`Qwen2Quantized`, `Qwen2Safetensors`) implementing D3; hardcoded chat template here. |
| D5 | CLI binary (`src/bin/`) | Args: model path/id + prompt -> load -> chat-template encode -> loop forward/argmax/decode -> print. **Gate:** coherent output; 15-token completion in the hundreds of ms. |
| D6 | Benchmark | Measure: load time, TTFT, tok/s, peak RSS. One small printed table. Feeds chunk size (Phase 3) + prefill strategy (Phase 4). |
| D7 | Tests | Encode<->decode round-trip; determinism (same seed + argmax -> same output) — needed later to test the mask. |

**First step (de-risk before anything else):** a tiny script that just
confirms `Qwen2.5-3B-Instruct-GGUF` loads via `from_gguf`. candle's tested
path is Qwen2, so Qwen2.5 GGUF (same `qwen2.*` metadata keys from llama.cpp)
*should* load but is unverified. If it fails, fall back to
`Qwen/Qwen2-1.5B-Instruct-GGUF`.

**Reading list (curated):**
1. `candle-examples/examples/qwen/main.rs` — canonical safetensors flow.
2. `candle-examples/examples/quantized-qwen2-instruct/main.rs` — GGUF path.
3. docs.rs `candle_transformers::generation::LogitsProcessor`.
4. `tokenizers` crate quickstart (HF) — only need `from_file`/`encode`/`decode`.
5. `hf-hub` sync API docs — `Api`/`Repo`/`repo.get`.
6. *(Optional, for the learning goal)* Parul Sharma, "Generation Sampling in
   LLMs" — the logits concept Phase 2 manipulates.

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
2. Eager prefill worker: after `text_ready`, prefill the fixed prefix (system
   prompt + instruction + the visible sentence) into the KV-cache in the
   background; emit `state: "interactive_ready"` when the hover path can
   answer in sub-50ms. On scroll to a new sentence: `clear` + re-prefill its
   prefix (no rollback in candle's opaque cache).

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

- **GGUF loading risk (de-risk first):** candle's tested GGUF path is Qwen2;
  a Qwen2.5 GGUF *should* load (`qwen2.*` metadata keys from llama.cpp) but is
  unverified. First Phase 1 script: does `quantized_qwen2::ModelWeights::from_gguf`
  load a Qwen2.5-3B GGUF? Fallback: `Qwen/Qwen2-1.5B-Instruct-GGUF`, then
  safetensors. Use Q8_0 for quality, Q4_K_M if RAM is tight.
- **Hungarian quality of Qwen2.5-3B:** acceptable for v1; if lemma/POS is
  unreliable, the mask can restrict lemma output to a small vocab later.
- **Exact token-range correctness:** tokenizer offsets are ground truth;
  model ranges are validated against them (drop-and-rerun fallback).
- **Second adapter:** the design goal is model-agnostic, but the first
  implementation pins Qwen2.5. Adding Llama-3.2 later is one new `Lm` impl —
  worth validating the trait stays clean when that happens.
