import { defineConfig, mergeConfig } from "vitest/config";
import viteConfig from "./vite.config";
import { normalizePath } from "vite";
import path from "node:path";

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      // The custom host renderer needs client SFC functions; SSR-only snapshot
      // tests keep their existing transform and do not need a DOM emulator.
      // Absolute paths also match checkouts inside hidden worktree folders.
      testTransformMode: {
        web: [
          "**/modelPicker.test.ts",
          "**/fontSettings.test.ts",
          "**/transcriptionDevice.test.ts",
          "**/libraryBrowser.test.ts",
          "**/workspaceTools.test.ts",
          "**/inlineChat.test.ts",
          "**/dictationControl.test.ts",
          "**/summaryFormatControls.test.ts",
          "**/summaryNavigation.test.ts",
          "**/readingTransport.test.ts",
        ].map((pattern) =>
          normalizePath(
            path.resolve(__dirname, "tests/unit", path.basename(pattern)),
          ),
        ),
      },
    },
  }),
);
