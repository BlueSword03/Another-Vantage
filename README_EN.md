# Another-Vantage

<p align="center">
  <a href="./README.md">简体中文</a> | <a href="./README_EN.md">English</a>
</p>

A tool that brings part of Lenovo Vantage to Linux — for Legion, IdeaPad and Think PCs.

**Status: planning; no code yet.** The first release is a **read-only CLI** (`--info` / `--config` / `--firmware`); writes come later.
The support matrix below reflects the development machine, **ThinkBook 16 G4+ IAP (21CY)**; other models are marked “unverified”.

## Usage

```bash
./another-vantage [--info [ITEM] | --config [ITEM] | --firmware] [OPTIONS]
```

```text
Groups (flags)
  --info        Read basic information
  --config      Read hardware configuration state (not a config file)
  --firmware    Check BIOS version / update availability

Items
  model  spec  performance  charge  lid  power  io

Options
  --json        Machine-readable output
  --no-color    Disable colors
  --verbose     Verbose logging
  --version
  --help
```

Notes:

- `--info` without an item prints everything; `--config` without an item prints all configuration states.
- Reads are implemented first; **writes are not implemented yet**. Planned syntax: `--set KEY=VALUE` (e.g. `--set performance=quiet`), plus `--dry-run` / `--yes`.
- Some reads require root: serial number (`product_serial`), BIOS current values (`thinklmi/*/current_value`), RAPL energy. On `EACCES` the CLI prints:

```text
Administrator privileges required: run as root or with sudo, e.g.:
  sudo another-vantage --info model
```

- All writes require root and must be verified by reading back.
- No kernel modules; no BIOS flashing from Linux.

Example:

```bash
➜  ./another-vantage --info model
Another Vantage ver 0.1.0
Device Model: ThinkBook 16 G4+ IAP
```

> The model comes from DMI `product_version`; the old example (`ThinkBook G4+ IAP`) was missing `16`.

## 21CY support matrix

> This is **interface feasibility**, not implementation status.
> ✅ available | ⚠️ partial / root required / to be verified | ❌ unavailable or unsupported on this machine | — unverified / model-dependent

| Feature | 21CY | Notes |
|---|---|---|
| Model / serial | ✅ / ⚠️ | Model is world-readable; serial is `0400`, root required |
| CPU model / cores / threads / HT | ✅ | i9-12900H, 14C/20T |
| Current CPU TDP | ⚠️ | RAPL reading needs a defined interpretation; `energy_uj` requires root |
| GPU model | ✅ | Iris Xe (no dGPU) |
| GPU shaders / TGP | ❌ | Not meaningful for an iGPU |
| Memory capacity | ✅ | 32 GB |
| Memory speed / part / form factor | ⚠️ | Requires `dmidecode`; may fail without root/lockdown |
| Drive capacity / model / interface | ✅ | 2× NVMe |
| Battery vendor / model / capacity / type / health | ✅ | LGES / L21L4PD8 / 65.48 Wh / Li-ion / 92.2% / 165 cycles |
| Screen size / model / resolution | ✅ | EDID: CSO 5644, 34×22 cm, 2560×1600 |
| Motherboard / BIOS / chipset | ✅ | LNVNB161216 / HYCN54WW |
| Audio / network model & features | ✅ | AX210 / I219-V |
| CPU temperature / utilization | ✅ | coretemp + `/proc/stat` |
| GPU temperature / utilization | ❌ | No iGPU temperature sysfs; utilization needs `intel_gpu_top`/PMU |
| Memory temperature | ❌ | No DIMM path |
| Drive temperature | ✅ | NVMe hwmon; `drivetemp` is for SATA |
| Swap | ✅ | 16 GB |
| Performance modes | ⚠️ | `platform_profile` 3 profiles + BIOS “Performance Enhanced” toggle |
| Charging modes | ✅ | `charge_types`: Fast / Standard / Long_Life |
| Lid-open power-on | ❌ | 21CY has no `flip_to_start` / `instant_boot` |
| USB charging while off | ✅ / ❌ | Toggle yes; **shut-off threshold unavailable** |
| I/O switches | ⚠️ | 10-item `thinklmi` allowlist; reading/writing requires root |
| Fan mode / fan curve | ⚠️ / ❌ | `fan_mode` exists, semantics to be verified; curve unavailable |
| dGPU mode / overclocking | ❌ | No dGPU, no Legion interfaces |
| BIOS update | ⚠️ | Check only; no LVFS path for 21CY |
| Manual TDP / power limits | ⚠️ | Root/MSR; unverified on this machine |
| Model picture | — | Planned; needs an asset mapping |
| Chinese docs / localization | ✅ | Chinese-first |

## Planned features

- [ ] Basic information
    - [ ] Model and serial (serial requires root)
    - [ ] Model picture (planned)
    - [ ] Specs
        - [ ] CPU model, cores, threads, HT status, current TDP (⚠️ definition TBD)
        - [ ] GPU model (Iris Xe here); shaders/TGP not applicable to iGPU
        - [ ] Memory speed, capacity, part number, form factor (⚠️ dmidecode)
        - [ ] Drive capacity, model, interface
        - [ ] Battery vendor, model, capacity, type, health
        - [ ] Screen size, model, resolution (EDID)
        - [ ] Motherboard, BIOS version, chipset
        - [ ] Audio model
        - [ ] Network model, feature support
    - [ ] Simple monitoring
        - [ ] CPU temperature and utilization
        - [ ] GPU temperature and utilization (⚠️ may be unavailable on iGPU)
        - [ ] Memory temperature (❌ unavailable here), free/used
        - [ ] Swap free/used
        - [ ] Drive temperature, free/used
- [ ] Configuration changes (**writes later**)
    - [ ] Performance modes: 3 `platform_profile` options; “Performance Enhanced” is a BIOS setting and may be invisible to Linux
    - [ ] Charging modes: `charge_types` (Standard / Long Life / Fast)
    - [ ] Lid-open power-on: model-dependent; unsupported on 21CY
    - [ ] USB charging while off (threshold unavailable)
    - [ ] I/O switches: `thinklmi` allowlist (USBPort / MemoryCardSlot / IntegratedCamera / Bluetooth / FingerprintReader / Microphone / EthernetLAN / Thunderbolt / WirelessLAN / AlwaysOnUSB)
- [ ] Advanced features on some models
    - [ ] dGPU-only / hybrid switch (Legion / dGPU models; not 21CY)
    - [ ] Preset overclocking (Legion / some models; not 21CY)
    - [ ] Fan curve (21CY only has coarse `fan_mode`; no curve)
- [ ] BIOS firmware version check (no flashing)
- [ ] Advanced
    - [ ] Manual TDP, GPU core/VRAM power limits and clocks (⚠️ risky; last)

## Building from source

Requires Rust stable and `cargo`. This session installed the toolchain under `.toolchain/` in the workspace (no system changes; see `docs/HANDOFF.md`).

```bash
cargo build --release
cargo test

./target/release/another-vantage --info model
./target/release/another-vantage --info spec --json
sudo ./target/release/another-vantage --info model   # serial and other privileged reads
```

The read-only CLI works today; writes, TUI, and GUI are not implemented yet.

## Design principles

1. **Capability probing first** — scan known interfaces, build a “what this machine supports” table; show explicit reasons for unsupported items.
2. **Read-only first** — stabilize `--info` / `--config` / `--firmware` before adding writes.
3. **Read back after writing** — every write is verified; if EC/firmware overrides it, say so.
4. **Least privilege** — normal reads need no root; privileged reads prompt for sudo; writes require root; no resident daemon.
5. **No kernel modules, no BIOS flashing, no arbitrary BIOS writes** — BIOS settings are allowlisted only.
6. **Frontend/data-layer decoupling** — CLI/TUI/GUI share the same capability model.

## Development status

- [x] 21CY hardware probing (`docs/probe-21CY.md`)
- [x] Competitor feature verification
- [x] Read-only CLI design and feature matrix
- [x] Language choice: Rust (GUI timing still open)
- [x] Phase 1: initial read-only CLI (`--info` / `--config` / `--firmware` + `--json`)
- [ ] Phase 2: monitoring
- [ ] Phase 3: writes
- [ ] Phase 4: TUI
- [ ] Phase 5: GUI
