import { Config } from "@remotion/cli/config";
import { enableTailwind } from "@remotion/tailwind";

Config.setVideoImageFormat("jpeg");
Config.setOverwriteOutput(true);

// Enable Tailwind CSS via official Remotion plugin
// Add GLB/GLTF asset support for @remotion/three model loading
Config.overrideWebpackConfig((config) => {
  const tailwindConfig = enableTailwind(config);
  tailwindConfig.module ??= { rules: [] };
  (tailwindConfig.module.rules ??= []).push({
    test: /\.(glb|gltf)$/,
    type: "asset/inline" as const,
  });
  return tailwindConfig;
});
