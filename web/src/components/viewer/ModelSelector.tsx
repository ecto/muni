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
      <button
        className="model-selector-btn"
        onClick={() => setIsOpen(!isOpen)}
        aria-expanded={isOpen}
        aria-haspopup="listbox"
      >
        <Cube size={18} weight="regular" aria-hidden="true" />
        <span>{currentModel?.name || "Select Model"}</span>
        <CaretDown size={12} weight="regular" className="chevron" aria-hidden="true" />
      </button>

      <div className="model-selector-menu" role="listbox" aria-label="Select a model">
        {models.map((section) => (
          <div key={section.section} className="model-menu-section" role="group" aria-label={section.section}>
            <div className="model-menu-section-title" id={`section-${section.section.toLowerCase().replace(/\s+/g, '-')}`}>
              {section.section}
            </div>
            {section.items.map((item) => (
              <button
                key={item.id}
                className={`model-menu-item ${currentModel?.id === item.id ? "active" : ""}`}
                onClick={() => {
                  onSelectModel(item);
                  setIsOpen(false);
                }}
                role="option"
                aria-selected={currentModel?.id === item.id}
              >
                <Cube size={18} weight="regular" aria-hidden="true" />
                <div className="model-menu-item-info">
                  <div className="model-menu-item-name">{item.name}</div>
                  <div className="model-menu-item-desc">{item.desc}</div>
                </div>
              </button>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
