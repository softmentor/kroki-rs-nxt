#!/usr/bin/env node

/**
 * Devflow path extension for Node.js projects using pnpm.
 *
 * Replaces the builtin Node extension to map Devflow commands to `pnpm`
 * instead of `npm`, supporting our pnpm workspace structure.
 */

import process from "node:process";

const CAPABILITIES = [
    "setup:deps",
    "setup:doctor",
    "fmt:check",
    "fmt:fix",
    "lint:static",
    "build:debug",
    "build:release",
    "test:unit",
    "test:integration",
    "test:smoke",
    "package:artifact"
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

    const action = (() => {
        switch (`${primary}:${selector}`) {
            case "setup:deps": return ["pnpm", "install"];
            case "setup:doctor": return ["pnpm", "--version"];
            case "fmt:check": return ["pnpm", "run", "fmt:check"];
            case "fmt:fix": return ["pnpm", "run", "fmt:fix"];
            case "lint:static": return ["pnpm", "run", "lint"];
            case "build:debug": return ["pnpm", "run", "build"];
            case "build:release": return ["pnpm", "run", "build"];
            case "test:unit": return ["pnpm", "run", "test:unit"];
            case "test:integration": return ["pnpm", "run", "test:integration"];
            case "test:smoke": return ["pnpm", "run", "test:smoke"];
            case "package:artifact": return ["pnpm", "pack"];
            default: return null;
        }
    })();

    if (action) {
        return { program: action[0], args: action.slice(1) };
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

    fail("usage: devflow-ext-node.mjs [--discover|--build-action]");
}

main();
