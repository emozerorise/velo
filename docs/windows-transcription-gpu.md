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

Compare inference time separately from total time and report cold and repeat
GPU runs separately. CPU/GPU text agreement is not an accuracy score without
a human-reviewed reference transcript. Do not relabel a CPU fallback as GPU.

## Hardware acceptance

Hardware: Intel Core i5-14400F (10 cores / 16 logical processors), RTX 5070.
Input: the same local 224.576-second M4A used to reproduce slow transcription.
Model: `ggml-large-v3-turbo-q5_0.bin` (574,041,195 bytes).
Timing and GPU-offload results will be recorded after the release binary runs
on this hardware; hosted CI has no RTX GPU and cannot certify performance.
