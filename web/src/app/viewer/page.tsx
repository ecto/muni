"use client";

import { useState, useCallback } from "react";
import dynamic from "next/dynamic";
import Link from "next/link";
import { ArrowLeft, Polygon, Tag } from "@phosphor-icons/react";
import * as THREE from "three";
import {
  ModelSelector,
  ComponentExplorer,
  Inspector,
  ScaleDropdown,
  findModelById,
  type ModelInfo,
  type ComponentInfo,
  type ViewerState,
} from "@/components/viewer";
import "./viewer.css";

// Dynamically import ModelViewer with SSR disabled to prevent pre-render issues
const ModelViewer = dynamic(
  () => import("@/components/viewer/ModelViewer").then((mod) => mod.ModelViewer),
  { ssr: false }
);

export default function ViewerPage() {
  const [state, setState] = useState<ViewerState>(() => ({
    modelInfo: findModelById("bvr1") || null,
    selectedComponent: null,
    labelsVisible: true,
    wireframe: false,
    modelSize: null,
    modelCenter: null,
  }));

  const [activeScales, setActiveScales] = useState<Set<string>>(new Set());

  const handleSelectModel = useCallback((model: ModelInfo) => {
    setState((prev) => ({
      ...prev,
      modelInfo: model,
      selectedComponent: null,
      modelSize: null,
      modelCenter: null,
    }));
  }, []);

  const handleModelLoaded = useCallback((size: THREE.Vector3, center: THREE.Vector3) => {
    setState((prev) => ({
      ...prev,
      modelSize: size,
      modelCenter: center,
    }));
  }, []);

  const handleSelectComponent = useCallback((component: ComponentInfo) => {
    setState((prev) => ({
      ...prev,
      selectedComponent: component,
    }));
  }, []);

  const handleToggleLabels = useCallback(() => {
    setState((prev) => ({
      ...prev,
      labelsVisible: !prev.labelsVisible,
    }));
  }, []);

  const handleToggleWireframe = useCallback(() => {
    setState((prev) => ({
      ...prev,
      wireframe: !prev.wireframe,
    }));
  }, []);

  const handleToggleScale = useCallback((id: string) => {
    setActiveScales((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const handleFocus = useCallback(() => {
    // TODO: Implement camera focus
  }, []);

  const handleIsolate = useCallback(() => {
    // TODO: Implement mesh isolation
  }, []);

  const handleReset = useCallback(() => {
    setState((prev) => ({
      ...prev,
      selectedComponent: null,
    }));
  }, []);

  const showExplorer = state.modelInfo?.hasLabels && state.labelsVisible;

  return (
    <div className="viewer-container">
      <ModelViewer
        state={state}
        activeScales={activeScales}
        onModelLoaded={handleModelLoaded}
        onSelectComponent={handleSelectComponent}
      />

      {/* Top bar */}
      <div className="top-bar">
        <div className="top-left">
          <Link href="/" className="back-link">
            <ArrowLeft size={12} weight="regular" />
            <span>Home</span>
          </Link>
          <ModelSelector currentModel={state.modelInfo} onSelectModel={handleSelectModel} />
        </div>
      </div>

      {/* Component explorer */}
      <ComponentExplorer
        visible={showExplorer || false}
        selectedComponent={state.selectedComponent}
        onSelectComponent={handleSelectComponent}
        onClose={handleToggleLabels}
      />

      {/* Inspector panel */}
      <Inspector
        visible={state.modelInfo !== null}
        modelInfo={state.modelInfo}
        modelSize={state.modelSize}
        selectedComponent={state.selectedComponent}
        onFocus={handleFocus}
        onIsolate={handleIsolate}
        onReset={handleReset}
      />

      {/* Bottom controls */}
      <div className="bottom-bar">
        <div className="controls-group">
          <button
            className={`control-btn ${state.wireframe ? "active" : ""}`}
            onClick={handleToggleWireframe}
          >
            <Polygon size={14} weight="regular" />
            Wireframe
          </button>
          <div className="control-divider" />
          <ScaleDropdown activeScales={activeScales} onToggleScale={handleToggleScale} />
          <div className="control-divider" />
          <button
            className={`control-btn ${state.labelsVisible ? "active" : ""}`}
            onClick={handleToggleLabels}
          >
            <Tag size={14} weight="regular" />
            Labels
          </button>
        </div>
      </div>

      {/* Help text */}
      <div className="help-text">
        Grid: 10mm cells, 100mm sections
        <br />
        Drag to rotate · Scroll to zoom
        <br />
        Right-drag to pan
      </div>
    </div>
  );
}
