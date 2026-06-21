---
name: vmlab
description: "Reference and processes for vmlab. A declarative QEMU/KVM VM-lab orchestrator: labs and virtual networks declared in WCL, reusable disk templates built locally or distributed over OCI registries, and guest automation written in wscript. Use when working with vmlab or answering questions about it."
wskill_schema_version: 1.1.0
allowed-tools: []
disallowed-tools: []
disable-model-invocation: false
---

# vmlab

A declarative QEMU/KVM VM-lab orchestrator: labs and virtual networks declared in WCL, reusable disk templates built locally or distributed over OCI registries, and guest automation written in wscript.

**Upstream version:** `1.0`. If the real upstream has moved past this, the skill may be stale — bump `topic.version` and re-verify (see the update workflow).

vmlab orchestrates single-host VM labs: labs (VMs + virtual networks) are declared in WCL (`vmlab.wcl`), disk templates are built and stored locally or distributed via OCI registries, and automation is written in wscript scripts that drive guests (power, exec, keystrokes, screen matching, OCR).

A two-tier daemon (supervisor `vmlabd` + one daemon per lab) is auto-started by the CLI. This skill captures the full reference as data; `docs/vmlab-prd.md` is the binding spec if anything here disagrees.

## Parameters

Values to pass when invoking this skill — reference them as `$ARGUMENTS`, `$1`, `$2`, … in the prompt.

| Parameter | Description | How to determine the value |
| --- | --- | --- |
| $ARGUMENTS | The vmlab topic, CLI subcommand, WCL attribute, or wscript API method to look up. | Take it from the user's request. If empty, summarise the reference and ask what they need. |

<Boundary>

**Always:**

- Run `vmlab validate` after editing `vmlab.wcl` and before `vmlab up`.

- For multi-step guest automation, write a wscript script (`vmlab script x.wscript`) instead of chaining many `vmlab exec` calls.

- Cite the exact reference page when answering.

**Ask first:**

- Which lab or template is meant when multiple `vmlab.wcl` files or store versions are plausible targets.

**Never:**

- Run `vmlab destroy` or `vmlab template rm` without explicit user say-so — both delete state (clones / store images).

- Invent WCL attributes or wscript functions: everything that exists is in the reference; if it's not there, check `src/config/schema.wcl` or `src/scripting/mod.rs` before using it.

</Boundary>

## Reference

- [Labs & networking](references/index_ix_labs.md) — Declare VMs and the virtual networks that connect them.

- [Templates & distribution](references/index_ix_templates.md) — Build reusable disk images and move them between machines.

- [Automation (wscript)](references/index_ix_automation.md) — Drive guests with wscript provision scripts and event handlers.

- [Operations & hosting](references/index_ix_operations.md) — Run vmlab: the CLI, daemons, host config, profiles, containers and WSL2.

- [CLI reference](references/cli_ref.md) — every `vmlab` subcommand, its arguments and switches.

- [Concepts](references/concepts_ref.md) — core ideas, one page each.

- [Entities](references/entities_ref.md) — concrete things in the topic.

- [Facts](references/facts_ref.md) — value tables and constants.

- [Processes](references/processes_ref.md) — task runbooks.

- [Glossary](references/glossary_ref.md) — terms and definitions.

- [Related skills](references/related_ref.md) — cross-references to other wskills.
