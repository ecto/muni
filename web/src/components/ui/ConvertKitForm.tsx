"use client";

import Script from "next/script";

export function ConvertKitForm() {
  return (
    <Script
      async
      data-uid="cb7ad78925"
      src="https://municipal-robotics.kit.com/cb7ad78925/index.js"
      strategy="lazyOnload"
    />
  );
}
