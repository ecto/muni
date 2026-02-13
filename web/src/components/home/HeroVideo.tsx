"use client";

import { useRef, useEffect } from "react";

export function HeroVideo() {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    // Ensure autoplay works (browsers require muted for autoplay)
    videoRef.current?.play().catch(() => {});
  }, []);

  return (
    <video
      ref={videoRef}
      className="landing-hero-video"
      autoPlay
      muted
      loop
      playsInline
      poster="/images/hype-reel-poster.jpg"
    >
      <source src="/videos/hype-reel.mp4" type="video/mp4" />
    </video>
  );
}
