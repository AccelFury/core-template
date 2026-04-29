// SPDX-License-Identifier: AGPL-3.0-or-later
const REQUIRED_DOCS = [
  "docs/00_overview.md",
  "docs/01_architecture.md",
  "docs/02_interface.md",
  "docs/03_microarchitecture.md",
  "docs/04_verification_plan.md",
  "docs/05_timing_and_constraints.md",
  "docs/06_resource_report.md",
  "docs/07_board_targets.md",
  "docs/08_release_checklist.md",
  "docs/09_portability_rules.md",
  "docs/10_commercial_licensing.md",
  "docs/11_fpga_family_notes.md",
  "docs/12_toolchain_notes.md",
  "docs/13_security_and_side_channels.md",
];

export async function checkDocs(root = Deno.cwd()): Promise<boolean> {
  for (const rel of REQUIRED_DOCS) {
    const p = `${root}/${rel}`;
    const stat = await Deno.stat(p).catch(() => null);
    if (!stat || !stat.isFile) {
      throw new Error(`missing documentation file: ${rel}`);
    }
    const text = await Deno.readTextFile(p);
    if (text.trim().length === 0) {
      throw new Error(`empty documentation file: ${rel}`);
    }
  }
  return true;
}
