import { Config } from "@remotion/cli/config";
import { enableTailwind } from "@remotion/tailwind";

Config.setVideoImageFormat("jpeg");
Config.setOverwriteOutput(true);

// Enable Tailwind CSS via official Remotion plugin
Config.overrideWebpackConfig((config) => {
  return enableTailwind(config);
});
