#!/usr/bin/env node

/**
 * Devflow path extension for host runtime dependencies and cleanup helpers.
 *
 * Capabilities:
 * - setup:deps          -> verify (default) or install host runtime dependencies
 * - setup:host-deps     -> verify (default) or install host runtime dependencies
 * - setup:prune-runs    -> prune GitHub Actions workflow runs
 * - setup:prune-containers -> prune local Podman/Docker objects
 * - setup:prune-deep    -> run all cleanup scopes (cache + runs + podman + temp)
 */

import process from "node:process";

const CAPABILITIES = [
  "setup:deps",
  "setup:host-deps",
  "setup:prune-runs",
  "setup:prune-containers",
  "setup:prune-deep",
];

function outputJson(value) {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(1);
}

function buildAction(cmdRef) {
  const primary = cmdRef?.primary ?? "";
  const selector = cmdRef?.selector ?? "";

  const requested = selector ? `${primary}:${selector}` : primary;
  if (!CAPABILITIES.includes(requested)) {
    process.exit(1);
  }

  if (requested.startsWith("setup:")) {
    if (selector === "prune-runs") {
      return { program: "bash", args: ["scripts/devflow-prune.sh", "--task", "runs"] };
    }
    if (selector === "prune-containers") {
      return { program: "bash", args: ["scripts/devflow-prune.sh", "--task", "containers"] };
    }
    if (selector === "prune-deep") {
      return { program: "bash", args: ["scripts/devflow-prune.sh", "--task", "deep"] };
    }

    const mode = process.env.KROKI_HOST_DEPS_MODE === "install" ? "install" : "check";
    const args = ["scripts/devflow-host-deps.sh", "--mode", mode];
    if (process.env.KROKI_INSTALL_CHROMIUM !== "1") {
      args.push("--without-chromium");
    }
    return { program: "bash", args };
  }

  process.exit(1);
}

function main() {
  if (process.argv.includes("--discover")) {
    outputJson(CAPABILITIES);
    return;
  }

  if (process.argv.includes("--build-action")) {
    let input = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => {
      input += chunk;
    });
    process.stdin.on("end", () => {
      const cmdRef = input.trim() ? JSON.parse(input) : {};
      outputJson(buildAction(cmdRef));
    });
    return;
  }

  fail("usage: devflow-ext-host-deps.mjs [--discover|--build-action]");
}

main();
