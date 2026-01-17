"use client";

import { useState, useRef } from "react";
import { Play, Pause } from "@phosphor-icons/react";

interface VideoPlayerProps {
  src: string;
  poster?: string;
  autoPlay?: boolean;
  muted?: boolean;
  loop?: boolean;
}

export function VideoPlayer({
  src,
  poster,
  autoPlay = false,
  muted = true,
  loop = true,
}: VideoPlayerProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [isPlaying, setIsPlaying] = useState(autoPlay);
  const [hasStarted, setHasStarted] = useState(autoPlay);

  const togglePlay = () => {
    if (!videoRef.current) return;

    if (isPlaying) {
      videoRef.current.pause();
      setIsPlaying(false);
    } else {
      videoRef.current.play();
      setIsPlaying(true);
      setHasStarted(true);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      togglePlay();
    }
  };

  return (
    <div
      className="video-player"
      onClick={togglePlay}
      onKeyDown={handleKeyDown}
      role="button"
      tabIndex={0}
      aria-label={isPlaying ? "Pause video" : "Play video"}
    >
      <video
        ref={videoRef}
        src={src}
        poster={poster}
        autoPlay={autoPlay}
        muted={muted}
        loop={loop}
        playsInline
        className="video-player-video"
        onPlay={() => setIsPlaying(true)}
        onPause={() => setIsPlaying(false)}
      />
      {!hasStarted && (
        <div className="video-player-overlay">
          <span className="video-player-button" aria-hidden="true">
            <Play size={64} weight="fill" />
          </span>
        </div>
      )}
      {hasStarted && !isPlaying && (
        <div className="video-player-overlay video-player-overlay-paused">
          <span className="video-player-button" aria-hidden="true">
            <Play size={48} weight="fill" />
          </span>
        </div>
      )}
    </div>
  );
}
