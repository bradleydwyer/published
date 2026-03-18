---
name: published
description: "Check if an app name is available on the Apple App Store and Google Play. Use when a user wants to check app name availability, find out if an app name is taken, research names for a new mobile app, or compare candidate app names across stores."
allowed-tools:
  - Bash(published:*)
user-invocable: true
argument-hint: "<name> [name2 name3 ...]"
metadata:
  author: bradleydwyer
  version: "0.1.0"
  status: experimental
---

# published -- App Store Name Availability Checker

Checks whether an app name is available on the Apple App Store and Google Play. Always use `-j` for JSON output.

## When to Use This Skill

- User wants to know if an app name is taken
- Brainstorming or comparing names for a new mobile app
- Checking a name on a specific store (App Store or Google Play)

## Installation

The `published` CLI must be available on PATH. Verify with `published --list-stores`. Install before proceeding if not found.

## Workflow

### Step 1: Determine Scope

| User Says | Flags |
|---|---|
| "is X available?" (general) | (none, checks both stores) |
| "check on App Store only" | `--stores app_store` |
| "check on Google Play only" | `--stores google_play` |
| multiple candidate names | pass all names as positional args |

### Step 2: Run the Check

Always use `-j` for JSON output:

```bash
published -j MyApp                          # both stores
published -j foo bar baz                    # multiple candidates
published -j --stores app_store MyApp       # App Store only
published -j --stores google_play MyApp     # Google Play only
```

### Step 3: Report Results

Parse the JSON and present a clear summary:
- Lead with the verdict: is the name available where it matters?
- Group results by available/taken
- If checking multiple names, compare them side by side
- Call out conflicts explicitly
- When choosing between candidates, recommend the name with the broadest availability

## CLI Quick Reference

```bash
published MyApp                   # both stores
published foo bar baz             # multiple names
published -j MyApp                # JSON output (always use this)
published -v MyApp                # verbose per-store detail
published --stores app_store MyApp  # specific store
published --list-stores           # show all supported stores
```

## Tips

- Always use `-j`. Human-readable output is for direct terminal use only.
- If a name is taken, suggest variations and check those too.
