"use client";

import { X } from "@phosphor-icons/react";
import { bvr1Components, type ComponentInfo } from "./ModelCatalog";

interface ComponentExplorerProps {
  visible: boolean;
  selectedComponent: ComponentInfo | null;
  onSelectComponent: (component: ComponentInfo) => void;
  onClose: () => void;
}

export function ComponentExplorer({
  visible,
  selectedComponent,
  onSelectComponent,
  onClose,
}: ComponentExplorerProps) {
  // Group components by section
  const sections: Record<string, ComponentInfo[]> = {};
  bvr1Components.forEach((c) => {
    if (!sections[c.section]) sections[c.section] = [];
    sections[c.section].push(c);
  });

  return (
    <div className={`component-explorer ${visible ? "" : "hidden"}`}>
      <div className="explorer-header">
        <span className="explorer-title">Components</span>
        <span className="explorer-count">{bvr1Components.length} parts</span>
        <button className="explorer-close" onClick={onClose} title="Hide labels">
          <X size={14} weight="regular" />
        </button>
      </div>
      <div className="explorer-content">
        {Object.entries(sections).map(([name, items]) => (
          <div key={name} className="explorer-section">
            <div className="explorer-section-title">{name}</div>
            {items.map((item) => (
              <div
                key={item.id}
                className={`explorer-item ${selectedComponent?.id === item.id ? "active" : ""}`}
                onClick={() => onSelectComponent(item)}
              >
                <div className="explorer-item-number">{item.id}</div>
                <div className="explorer-item-info">
                  <div className="explorer-item-name">{item.name}</div>
                  <div className="explorer-item-desc">{item.desc}</div>
                </div>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
