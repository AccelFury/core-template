// SPDX-License-Identifier: AGPL-3.0-or-later

# AccelFury FPGA IP Template

## What this template is

`accelfury-fpga-ip-template` is a GitHub template for production-style FPGA IP
development with strict portability boundaries for reusable reusable modules.

Template focus:

- reusable RTL-first architecture
- portable core only in `rtl/core/` and board-specific integration outside core
- zero Python, zero C++ testbench, zero Node/npm dependency
- first ready IP: `af_mod_add` (`result = (a + b) mod p`)

## No-Python policy

This repository contains no Python tooling and no Python reference
implementation:

- no `*.py`, no `*.pyc`, no `*.pyo`
- no `requirements.txt`, `pyproject.toml`, `setup.py`, `setup.cfg`, `tox.ini`,
  `poetry.lock`, `Pipfile`, `Pipfile.lock`, `.python-version`
- no `package.json`, no `node_modules`
- no cocotb
- no FuseSoC as mandatory dependency

Allowed implementation stack:

- Verilog-2001 / Verilog-2005 / SystemVerilog for RTL and testbench
- Rust for reference model, vector generation, host CLI/runner, reports
- TypeScript through Deno for metadata/docs/registry checks

## Language stack

- **Verilog-2001 / Verilog-2005 / SystemVerilog**: RTL, stream wrappers, tests,
  constraints docs
- **Rust**: reference arithmetic, deterministic vectors, CLI commands
  (`cargo xtask`), simulation orchestration
- **TypeScript (Deno)**: repo checks, manifest/registry validation, registry
  matrix, no-Python audit

Verilog generation policy:

- `cargo xtask new-ip ... --language verilog-2001` → `.v` RTL
- `cargo xtask new-ip ... --language verilog-2005` → `.v` RTL
- `cargo xtask new-ip ... --language systemverilog` (default) → `.sv` RTL
- The shipped `af_mod_add` has a SystemVerilog implementation in `rtl/core/` and
  a lowest-common-denominator Verilog-2001 implementation in
  `rtl/core/verilog2001/` that is also valid Verilog-2005.

## Quick start

```bash
cargo xtask vectors
cargo xtask sim-verilator
cargo xtask lint-rtl
cargo xtask sim-verilator --language verilog-2001
cargo xtask lint-rtl --language verilog-2005
cargo xtask check-spdx
cargo xtask check-docs
cargo xtask check-boards
cargo xtask check-no-python
deno task audit:repo
cargo xtask board-matrix
cargo xtask synth-gowin --board sipeed_tang_primer_20k_dock
cargo xtask synth-quartus --board terasic_de0_nano_cyclone4
cargo xtask synth-vivado --board digilent_arty_a7
cargo xtask synth-lattice-oss --board icebreaker_ice40up5k
```

## Status table

| Component               | Status                             |
| ----------------------- | ---------------------------------- |
| RTL architecture        | ready                              |
| Reference model         | ready                              |
| Deterministic vectors   | ready                              |
| Verilator simulation    | ready (`--binary` via Rust runner) |
| Lint and checks         | implemented                        |
| Board matrix            | ready                              |
| Synthesis templates     | placeholders by vendor flow        |
| Timing/resource reports | collection hooks ready             |

## Supported board matrix

- **Sipeed (Gowin)**: 10 boards (`sipeed_tang_nano_1k`, `sipeed_tang_nano_4k`,
  `sipeed_tang_nano_9k`, `sipeed_tang_nano_20k`, `sipeed_tang_primer_20k_core`,
  `sipeed_tang_primer_20k_dock`, `sipeed_tang_primer_20k_lite`,
  `sipeed_tang_primer_25k`, `sipeed_tang_mega_138k`,
  `sipeed_tang_mega_138k_pro`)
- **Lattice iCE40 / ECP5**: `icebreaker_ice40up5k`, `tinyfpga_bx_ice40lp8k`,
  `icestick_ice40hx1k`, `ulx3s_ecp5`, `orangecrab_ecp5`,
  `colorlight_5a_75b_ecp5`
- **Intel/Altera**: `terasic_de0_nano_cyclone4`, `terasic_de10_lite_max10`,
  `terasic_de10_nano_cyclonev_soc`, `qmtech_cyclone4_core`
- **Xilinx**: `digilent_basys3_artix7`, `digilent_arty_a7`, `digilent_cmod_a7`,
  `qmtech_artix7`, `ebaz4205_zynq7010`
- **Efinix**: `trion_t8`, `trion_t20`, `titanium_ti60`
- **Microchip**: `polarfire_splash_template`, `igloo2_template`

## FPGA family abstraction

See `registries/fpga_families.json`:

- vendor/family entries for Gowin, Lattice, Intel, Xilinx, Efinix, Microchip,
  `generic_asic_like`
- constraint support list and portability notes
- suggested toolchain mapping and resource behavior

## Toolchain abstraction

- `toolchains/gowin_eda`
- `toolchains/intel_quartus`
- `toolchains/lattice_oss` (iCE40 and ECP5 open flows)
- `toolchains/xilinx_vivado`
- `toolchains/lattice_radiant`
- `toolchains/efinix_efinity`
- `toolchains/microchip_libero`

## Portability rules

1. Core RTL has no vendor primitives and no board pins.
2. Board clocks/reset/I/O wrappers are outside core.
3. Valid/ready stream interfaces, optional simple `regmap/axi-lite/wishbone`,
   and optional UART/SPI command shell.
4. No gated clocks in core (use enables + sync reset strategy).
5. No initial-block RTL state, no board-specific memory/IP hard dependencies.
6. Assertions use Verilator-compatible formal checks.

## How to add a new IP (`af_ntt_butterfly`)

1. Copy `rtl/core/af_mod_add.sv` as template and rename module.
2. Preserve the same handshake contract (`i_valid`, `o_ready`, `o_valid`,
   `i_ready`).
3. Add RTL source files with target language:
   - `rtl/core/<new_ip>.sv` + `<new_ip>_top.sv` for SystemVerilog
   - `rtl/core/<new_ip>.v` + `<new_ip>_top.v` for Verilog-2001/Verilog-2005
4. Add deterministic edge/random vectors in `vectors/<new_ip>_*.json` and
   `.svh`.
5. Update `ip.manifest.json` fields (`source_files`, `interfaces`, `vectors`,
   `boards`).
6. Update docs under `docs/` and add optional board support notes.
7. Register new crate/module in `registries/ip_categories.json` if needed.
8. Run:

```bash
cargo xtask vectors
cargo xtask sim-verilator
cargo xtask lint-rtl
cargo xtask sim-verilator --language verilog-2001
cargo xtask lint-rtl --language verilog-2005
deno task audit:repo
```

Language-aware variant generation:

```bash
cargo xtask new-ip --name af_ntt_butterfly --category ntt_fft
cargo xtask new-ip --name af_ntt_butterfly --category ntt_fft --language verilog-2001
```

## How to add a new board

1. Add metadata in `registries/boards.registry.json`.
2. Add folder `boards/<vendor>/<board_id>/` with:
   - `README.md`
   - `bringup.md`
   - `board.status.json`
   - `top/af_board_top.sv`
   - `constraints/README.md`
   - constraint placeholder (`pins.cst`, `pins.pcf`, `constraints.lpf`,
     `project.qsf`, `constraints.xdc`, or `constraints.pdc`)
   - toolchain build script under `gowin/`, `lattice/`, `quartus/`, `vivado/`,
     `efinity/`, or `libero/`
3. Run `deno task check:boards` and `cargo xtask board-matrix`.

## How to add a new vendor

1. Add vendor toolchain adapter under `toolchains/<vendor>_template/` and
   documented flow.
2. Add family entries in `registries/fpga_families.json`.
3. Add at least one board and its status/readme template.
4. Extend checks to validate new board constraints/build files.

## How to add a new toolchain

1. Add toolchain adapter under `toolchains/<chain>/`.
2. Add entry to `registries/toolchains.registry.json` and CLI path mapping in
   `crates/xtask/src`.
3. Update CI workflow to call corresponding `cargo xtask synth-*` command.

## Licensing summary

- RTL / gateware: `CERN-OHL-S-2.0`
- Rust / TypeScript tooling: `AGPL-3.0-or-later`
- documentation: `CC-BY-SA-4.0`

## Commercial licensing note

Repository is open-licensed by default. Separate commercial terms exist for
closed-source bitstreams, ASIC handoff services, or SaaS/prover
commercialization. See `COMMERCIAL.md`.

## Release checklist

1. Update `ip.manifest.json` and vector metadata hash.
2. Validate `cargo xtask vectors`, `cargo xtask sim-verilator`,
   `cargo xtask lint-rtl`.
3. Run `deno task audit:repo` and `cargo xtask check-spdx`.
4. Ensure board registry and toolchain registry pass checks.
5. Update `CHANGELOG.md` with API/behavior changes.
6. Tag release and publish release artifacts in `reports/`.
