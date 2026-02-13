#!/usr/bin/env bash
# Generate AI shots via fal.ai Flux Pro v1.1
set -euo pipefail

FAL_KEY="09e24110-3629-4ccd-ad52-bb3996334b1c:ec90846f5ad6964d29d4e5d740e80ceb"
OUT_DIR="$(dirname "$0")/../public/reel/shots"
mkdir -p "$OUT_DIR"

API="https://fal.run/fal-ai/flux-pro/v1.1"

generate() {
  local filename="$1"
  local prompt="$2"

  if [[ -f "$OUT_DIR/$filename" ]]; then
    echo "SKIP $filename (exists)"
    return
  fi

  echo "GENERATING $filename..."
  local response
  response=$(curl -s -X POST "$API" \
    -H "Authorization: Key $FAL_KEY" \
    -H "Content-Type: application/json" \
    -d "$(jq -n \
      --arg prompt "$prompt" \
      '{
        prompt: $prompt,
        image_size: "landscape_16_9",
        num_images: 1,
        output_format: "jpeg",
        enable_safety_checker: false,
        safety_tolerance: 6
      }'
    )")

  local url
  url=$(echo "$response" | jq -r '.images[0].url // empty')
  if [[ -z "$url" ]]; then
    echo "FAIL $filename: $(echo "$response" | jq -r '.detail // .message // "unknown error"')"
    return 1
  fi

  curl -s -o "$OUT_DIR/$filename" "$url"
  echo "OK $filename ($(du -h "$OUT_DIR/$filename" | cut -f1))"
}

# Run all generations
generate "snow-problem-2.jpg" \
  "Wheelchair user stranded on unshoveled snowy sidewalk, residential neighborhood, overcast winter, photojournalistic, 16:9"

generate "flex-campus.jpg" \
  "Compact open-frame matte-black autonomous rover with diagonal steel cross-bracing, four chunky pneumatic hub-motor wheels at corners, exposed batteries and electronics visible inside skeletal frame, driving on snowy university campus sidewalk, modern buildings background, winter golden hour, cinematic, 16:9"

generate "flex-night-ops.jpg" \
  "Compact matte-black open-frame rover with four hub-motor wheels, diagonal steel bracing, exposed internals, bright LED headlights on, clearing snow from sidewalk at night, snow falling, dramatic light beams, residential street, cinematic, 16:9"

generate "flex-plow-closeup.jpg" \
  "Macro close-up of metal plow blade pushing through fresh snow on concrete sidewalk, snow spraying, shallow DOF, dramatic lighting, 16:9"

generate "flex-multi-rover.jpg" \
  "Fleet of 5 compact matte-black open-frame rovers with diagonal steel bracing and hub-motor wheels, formation on snowy suburban sidewalk, aerial drone view, winter morning, cinematic, 16:9"

generate "tech-jetson.jpg" \
  "NVIDIA Jetson Orin NX compute module mounted inside open-frame robot chassis, green PCB, heatsink, ethernet cables, orange XT connectors, close-up, shallow DOF, workshop lighting, 16:9"

generate "biz-cities.jpg" \
  "Aerial view of snowy midwestern American city, roads cleared but sidewalks still covered in snow, winter, cinematic drone shot, 16:9"

generate "pink-bvr1.jpg" \
  "Compact open-frame autonomous rover with diagonal steel cross-bracing and four hub-motor wheels, but painted HOT PINK instead of black, sitting on snowy sidewalk, dramatic lighting, fashion editorial style, 16:9"

echo ""
echo "Done! Generated shots in $OUT_DIR"
ls -la "$OUT_DIR"
