"use client";

import type { ModelInfo, ComponentInfo } from "./ModelCatalog";
import * as THREE from "three";

interface InspectorProps {
  visible: boolean;
  modelInfo: ModelInfo | null;
  modelSize: THREE.Vector3 | null;
  selectedComponent: ComponentInfo | null;
  onFocus: () => void;
  onIsolate: () => void;
  onReset: () => void;
}

export function Inspector({
  visible,
  modelInfo,
  modelSize,
  selectedComponent,
  onFocus,
  onIsolate,
  onReset,
}: InspectorProps) {
  const title = selectedComponent?.name || modelInfo?.name || "Select a model";
  const subtitle = selectedComponent?.desc || modelInfo?.desc || "";

  return (
    <div className={`inspector ${visible ? "" : "hidden"}`}>
      <div className="inspector-header">
        <div className="inspector-title">{title}</div>
        <div className="inspector-subtitle">{subtitle}</div>
      </div>
      <div className="inspector-props">
        {selectedComponent ? (
          <>
            <div className="inspector-prop">
              <span className="inspector-prop-label">Specs</span>
              <span className="inspector-prop-value">{selectedComponent.specs}</span>
            </div>
            <div className="inspector-prop">
              <span className="inspector-prop-label">Section</span>
              <span className="inspector-prop-value">{selectedComponent.section}</span>
            </div>
          </>
        ) : modelSize ? (
          <div className="inspector-prop">
            <span className="inspector-prop-label">Dimensions</span>
            <span className="inspector-prop-value">
              {modelSize.x.toFixed(0)} × {modelSize.y.toFixed(0)} × {modelSize.z.toFixed(0)} mm
            </span>
          </div>
        ) : null}
      </div>
      <div className="inspector-actions">
        <button className="inspector-btn" onClick={onFocus}>
          Focus
        </button>
        <button className="inspector-btn" onClick={onIsolate}>
          Isolate
        </button>
        <button className="inspector-btn" onClick={onReset}>
          Reset
        </button>
      </div>
    </div>
  );
}
