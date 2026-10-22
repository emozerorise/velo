# Windows GPU transcription

Windows builds enable whisper-rs 0.16.0's Vulkan backend. The default context
uses the GPU when whisper.cpp can initialize a compatible device, retaining
its CPU fallback. macOS builds keep Metal. The model, beam size (5), chunk
length (300 seconds), language and vocabulary prompt behavior are unchanged.
The bundled application-local Vulkan loader also serves the Whisper backend;
no CUDA runtime or toolkit is required on the user's machine.

## Build and verify

Install Vulkan SDK 1.4.363.0 alongside the existing Windows build prerequisites.
Set `VULKAN_SDK` to the SDK root and add its `Bin` directory to the build PATH.
Use `CMAKE_GENERATOR=Ninja` from an MSVC developer environment; MSBuild file
tracking exceeds Windows path limits in the nested Vulkan shader generator.
Set `CARGO_TARGET_DIR` to a short absolute path (for example `C:/vt`) as well:
the explicit release target otherwise exceeds MSVC PDB path limits even with Ninja.
The Windows Preview workflow pins the official SDK installer version and SHA-256,
runs format/clippy/tests, builds the NSIS bundle, and checks the installed app
with a clean PATH. A regression test rejects CPU-only Windows builds.

## Reproduce the local benchmark

Build the release example after preparing libmpv and the frontend:

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --release --target x86_64-pc-windows-msvc --example transcription_bench
```

The workflow also uploads `windows-transcription-benchmark-x64`, containing
the executable, runtime DLLs and build revision. Extract it to a separate
local folder. Use the same original media and already-downloaded model for
each run, without playing media or running another transcription concurrently:

```powershell
./transcription_bench.exe "$media" "$model" cpu cpu.json
./transcription_bench.exe "$media" "$model" gpu gpu.json
```

Both modes call Velo's production extraction and transcription code with
language `auto`, an empty vocabulary prompt and beam size 5. CPU mode explicitly
disables GPU use in the context. GPU mode requires an enumerated Vulkan device,
and its log must also show Vulkan model/compute buffers. Check actual device
activity with `nvidia-smi`; enumeration alone does not prove offloading.
The example does not read/write the application's cache or contact a provider.
It refuses to overwrite existing output and validates nonempty segments and
ordered timestamps within the source duration. The total timer starts before GPU enumeration, because enumeration also creates
the Vulkan device and loads shaders. GPU setup is recorded separately; inference
time includes model initialization but excludes that device setup. Results include both extraction
and inference times, model, system capabilities, detected language and segments.
Keep these reports local: they contain the recording's transcript.

Compare inference time separately from total time and report first and repeat
GPU runs separately without assuming driver caches are cold. CPU/GPU text agreement is not an accuracy score without
a human-reviewed reference transcript. Do not relabel a CPU fallback as GPU.

## Hardware acceptance

Measured on 2026-10-22: Intel Core i5-14400F (10 cores / 16 logical processors),
NVIDIA GeForce RTX 5070, driver 621.01. The same private M4A reports 224.576
seconds in its container; production extraction yields 225.280 seconds of mono
16 kHz PCM in all three runs. Model: `ggml-large-v3-turbo-q5_0.bin`
(574,041,195 bytes), language `auto`, empty prompt, beam size 5, 300-second
chunks and 15 transcription threads.

Both modes used the same release executable, source `cbb72bae58fe247633d2cbde0aadd5a2e14f9ddb`,
built from PR merge commit `a78cd24db408107b09cf3bc5972ac19989bcad96`. Existing Preview CPU
flags were retained: `GGML_NATIVE`, `GGML_AVX`, `GGML_AVX2`, `GGML_FMA` and
`GGML_F16C` were all `OFF`. These ratios describe this Preview build on this
recording. There was one full CPU run and two fresh-process GPU runs; OS and
driver caches were not cleared.

| Mode | GPU setup (s) | Extraction (s) | Transcription incl. model (s) | Pipeline total (s) | CPU / total |
| :--- | ---: | ---: | ---: | ---: | ---: |
| CPU | 0.00 | 0.08 | 2266.28 | 2266.37 | 1.00x |
| RTX 5070, first run | 0.17 | 0.08 | 32.57 | 32.82 | 69.05x |
| RTX 5070, repeat run | 0.18 | 0.08 | 11.95 | 12.22 | 185.49x |

The measured region starts after executable/logging setup and ends after
transcription, before validation and JSON writing. It includes device setup,
extraction and model initialization, and excludes executable loading and process
teardown. Normal desktop background apps and two-second `nvidia-smi` sampling
remained active. The installer archive was downloaded and verified during the
CPU run; no other Velo transcription or playback was running.

Both GPU logs show a 573.40 MB `Vulkan0` model allocation and `using Vulkan0
backend`. Sampled GPU utilization peaked at 76% on the first run and 35% on the
repeat, with total device memory peaking at 2,610 / 2,628 MiB respectively
(including desktop use). CPU mode explicitly logged `use gpu = 0` and CPU model
buffers; its sampled GPU utilization peaked at 9% with at most 1,355 MiB used.
The GPU runs produced identical text/timestamps across 49 Thai segments. CPU
produced 36 Thai segments. Output is nonempty with ordered timestamps within the
extracted duration; no human reference transcript was used to score accuracy.

The SHA-256-verified NSIS package was also installed as Preview 0.9.0-beta.2 on
the same PC using update mode. Its **Transcribe only** action on the original
file persisted 49 Thai segments with the same model and empty prompt about
31.81 seconds after the click; the result UI was observed ready
within 50 seconds. The original model and library remained available. No summary
request was made. Raw recordings and transcript JSON reports remain local.

[Windows checks, bundle build, installed-app smoke test and artifacts](https://github.com/emozerorise/velo/actions/runs/39807521868)
and [frontend/macOS checks](https://github.com/emozerorise/velo/actions/runs/39807522035) passed for the tested
source revision. The benchmark artifact records both the PR head and actual
merge revision, and local reports record media/model/executable SHA-256 hashes.


## Processor selection and CPU fallback

Settings → Transcription offers Automatic (default), GPU and CPU for new file
transcriptions. Both Automatic and GPU prefer a supported GPU and retry once on
CPU if discovery, initialization, inference or the worker process fails. CPU
selects CPU directly. The choice is persisted; older settings default to Auto.
A running job keeps its original choice. Existing cached transcripts are not
rewritten. Microphone dictation retains its existing independent adapter.

Native computation runs in a child of the same packaged executable, entered
before Tauri initialization with a private worker argument. PCM, vocabulary and
transcript frames use anonymous pipes and do not become temporary transcript
files. The parent accepts a complete result only after successful child exit;
it kills/reaps workers on cancellation and never retries a cancelled job. Native
C++ exceptions, driver crashes and teardown failures are thereby contained.
CPU failure ends the job without replacing the preceding transcript cache.

GPU discovery uses the native registry's guarded initialization, not Vulkan's
high-level buffer enumeration. Whisper's synchronous initialization diagnostics
confirm that the selected context did not silently fall back to CPU; a failed
backend initialization clears the optimistic 'using backend' marker. This is
coupled to the pinned whisper-rs/sys version and covered by a regression test.

The reading workspace shows the selected processor and a persistent explanation
for CPU fallback; a six-second toast also announces missing/failed GPU. The same
notice is visible during transcribe-and-summarise. Settings writes are serialized
so an overlapping modal/job save cannot restore an older processor choice.

`transcription_bench MEDIA MODEL auto OUTPUT.json` runs the app's process-isolated
Auto path; `gpu-fallback` tests the GPU preference with CPU recovery, `cpu` runs
the app's CPU worker, and `gpu` remains a strict direct GPU diagnostic. Optional
`MAX_SECONDS` limits extracted samples for CPU/fallback regression runs and is
recorded in JSON; such runs are not full-file speed comparisons. Historic timing
above predates process isolation and is not re-labelled as the new revision.


## Process-isolated processor-mode acceptance

The processor controls and worker path were tested on the same RTX 5070 PC and
private source/model described above, using source
`847eedc96ea3de3e5be4d7d247a1da30ebcf1433`, built at PR merge revision
`ffce8f34fff57a284babc64904f03053d8965692`. Model, prompt, beam, chunk and CPU
build flags were unchanged. GPU/OS caches were not cleared. The existing desktop
app remained open and idle; no concurrent transcription or playback was started.

| Preference / condition | Samples (s) | Actual execution | Pipeline total (s) | Segments |
| :--- | ---: | :--- | ---: | ---: |
| Automatic | 225.280 | GPU | 12.14 | 49 |
| GPU | 225.280 | GPU | 12.08 | 49 |
| CPU | 6.000 | CPU | 289.73 | 2 |
| Automatic, GPU hidden | 6.000 | CPU, `no_gpu` | 290.64 | 2 |

Both full GPU outputs have identical text/timestamps to each other and to the
preceding GPU revision. The CPU and no-GPU outputs also match each other. These
are functional checks, not accuracy measurements. The six-second CPU runs are
not full-file speed comparisons; Whisper processes a padded window and this
Preview retains generic CPU build flags.

The no-GPU run used invalid `VK_DRIVER_FILES` and `VK_ICD_FILENAMES` only for a
non-elevated test process and its children. Its worker recorded `use gpu = 0`
and reported `no_gpu`. An earlier elevated attempt ignored these loader overrides,
used GPU, and is excluded from no-GPU acceptance. Driver failure recovery was
also tested with controlled worker exits in subprocess tests; a real NVIDIA
driver crash was not injected.

Validation at this source: 310 frontend tests plus typecheck/lint passed;
[Windows native checks and NSIS installed-app smoke test](https://github.com/emozerorise/velo/actions/runs/39821729169)
and [frontend/macOS native checks](https://github.com/emozerorise/velo/actions/runs/39821729152)
passed. Windows library tests reported 113 passed / 1 ignored; macOS reported
112 passed / 1 ignored, with integration suites also passing.

The new NSIS SHA-256 was verified locally. Before the approved local update,
the visible in-memory chat, existing program, profile JSON and transcript cache
were backed up. Windows Code Integrity then blocked installer startup (event
3077, enterprise signing policy), leaving the installed app unchanged. The old
app was reopened. New settings persistence and fallback notices have automated
coverage, but the new installed settings UI has not been manually accepted on
this PC. No signing policy was changed. Raw recordings, chat and transcript
reports remain local; these acceptance runs did not invoke a summary provider.
