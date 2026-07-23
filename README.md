# Sabotage at T-Minus

A game built with [Bevy](https://bevy.org) and
[Rapier](https://rapier.rs) for 2D physics. Rust toolchain and dev tooling are
pinned with [mise](https://mise.jdx.dev); CI builds for web, Linux, Windows and
macOS and publishes to [itch.io](https://itch.io).

## Getting started

```sh
mise trust     # once, to allow this repo's mise.toml
mise install   # rust, wasm-bindgen-cli, butler
mise run       # list every task
mise run run   # play the game
```

On Linux you also need Bevy's system dependencies — on Arch:

```sh
sudo pacman -S --needed alsa-lib systemd-libs vulkan-icd-loader
```

### Faster iteration

```sh
cargo run --features dev
```

The `dev` feature links Bevy dynamically, so an edit to `src/` relinks in
seconds instead of recompiling the engine. It also turns on Rapier's debug
renderer, which draws every collider as a wireframe. Never ship a build made
with it — the binary depends on `libbevy_dylib.so` sitting next to it.

## Tasks

| Task                        | What it does                                          |
| --------------------------- | ----------------------------------------------------- |
| `mise run run`              | Run natively                                          |
| `mise run fmt` / `fmt:check`| Format / verify formatting                            |
| `mise run lint`             | Clippy, warnings as errors                            |
| `mise run test`             | Test suite                                            |
| `mise run ci`               | Everything CI runs, locally                           |
| `mise run build:native`     | Release build for this machine                        |
| `mise run build:web`        | wasm + JS glue + assets into `dist/web`               |
| `mise run serve:web`        | Build for web and serve on <http://localhost:8080>    |
| `mise run publish:web`      | Build for web and `butler push` it to itch.io         |
| `mise run clean`            | Drop `target/` and `dist/`                            |

## Shipping

### One-time setup

1. **Create the itch.io page.** On <https://itch.io/game/new>, set the URL slug
   to `sabotage-at-t-minus` under the user `tanayseven`, and set *Kind of
   project* to **HTML** so the web build plays in the browser. Leave it as a
   draft until you're ready.
2. **Get a butler API key** from <https://itch.io/user/settings/api-keys>.
3. **Add it to GitHub** as a repository secret named `BUTLER_API_KEY`
   (`gh secret set BUTLER_API_KEY`).
4. For local pushes, log in once with `mise exec -- butler login`.

### Releasing

```sh
git tag v0.1.0
git push origin v0.1.0
```

The `Release` workflow builds all four platforms, pushes each to its itch.io
channel (`html5`, `linux`, `windows`, `mac`) tagged with the version from the
tag, and attaches the archives to a GitHub release. `workflow_dispatch` does
the same thing with a version you type in, skipping the GitHub release.

Once the `html5` channel has a build, tick *This file will be played in the
browser* on the itch.io page and set the viewport to match your game.

## Layout

```
src/           game code
assets/        runtime assets, copied next to the binary in every build
web/index.html shell page for the wasm build
mise.toml      toolchain pins, env, and every task
.github/       CI and the release/publish pipeline
```

## Notes

- `bevy_rapier2d` is pinned to a version built against this Bevy release; the
  two have to be bumped together, and the crate's `CHANGELOG.md` is the place
  to check which Bevy version a given release targets.
- Physics runs in world units, with `PIXELS_PER_METER` in `src/main.rs` telling
  Rapier how those units map onto the metre-scale its solver is tuned for.
  Gravity is set explicitly rather than left at Rapier's default, because a
  realistic 1g falls too slowly to read as a platformer.
- `wasm-bindgen-cli` in `mise.toml` must match the `wasm-bindgen` version in
  `Cargo.lock`. When you bump Bevy, check the lockfile and update the pin.
- Release builds use `lto = "thin"` and `codegen-units = 1`; the web profile
  (`wasm-release`) additionally optimises for size, since download time is the
  thing players actually feel.
- The `Warm build cache` workflow rebuilds Bevy and Rapier on `main` whenever
  the dependency set changes, so the tag-triggered `Release` builds restore a
  warm cache instead of compiling the engine from scratch (~40 min → a few).
  Its four jobs mirror the Release build jobs' ids, cache keys, runners, and
  build commands on purpose — GitHub scopes caches by git ref, so a warm cache
  only reaches a tag run if it was written on the default branch under the same
  job id and key. Keep the two in lockstep, or releases silently go cold again.
