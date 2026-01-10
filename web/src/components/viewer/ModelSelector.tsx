"use client";

import { useState, useRef, useEffect } from "react";
import { Cube, CaretDown } from "@phosphor-icons/react";
import { models, type ModelInfo } from "./ModelCatalog";

interface ModelSelectorProps {
  currentModel: ModelInfo | null;
  onSelectModel: (model: ModelInfo) => void;
}

export function ModelSelector({ currentModel, onSelectModel }: ModelSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }

    document.addEventListener("click", handleClickOutside);
    return () => document.removeEventListener("click", handleClickOutside);
  }, []);

  return (
    <div className={`model-selector ${isOpen ? "open" : ""}`} ref={containerRef}>
      <button className="model-selector-btn" onClick={() => setIsOpen(!isOpen)}>
        <Cube size={18} weight="regular" />
        <span>{currentModel?.name || "Select Model"}</span>
        <CaretDown size={12} weight="regular" className="chevron" />
      </button>

      <div className="model-selector-menu">
        {models.map((section) => (
          <div key={section.section} className="model-menu-section">
            <div className="model-menu-section-title">{section.section}</div>
            {section.items.map((item) => (
              <div
                key={item.id}
                className={`model-menu-item ${currentModel?.id === item.id ? "active" : ""}`}
                onClick={() => {
                  onSelectModel(item);
                  setIsOpen(false);
                }}
              >
                <Cube size={18} weight="regular" />
                <div className="model-menu-item-info">
                  <div className="model-menu-item-name">{item.name}</div>
                  <div className="model-menu-item-desc">{item.desc}</div>
                </div>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
