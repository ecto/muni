"use client";

import { useState, useRef, useEffect } from "react";
import { Ruler, Plant, PersonArmsSpread, StarFour } from "@phosphor-icons/react";
import { scaleReferences } from "./ModelCatalog";

const iconMap: Record<string, React.ElementType> = {
  plant: Plant,
  "person-arms-spread": PersonArmsSpread,
  "star-four": StarFour,
};

interface ScaleDropdownProps {
  activeScales: Set<string>;
  onToggleScale: (id: string) => void;
}

export function ScaleDropdown({ activeScales, onToggleScale }: ScaleDropdownProps) {
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

  const hasActive = activeScales.size > 0;

  return (
    <div className={`scale-dropdown ${isOpen ? "open" : ""}`} ref={containerRef}>
      <button
        className={`control-btn ${hasActive ? "active" : ""}`}
        onClick={() => setIsOpen(!isOpen)}
      >
        <Ruler size={14} weight="regular" />
        Scale
      </button>
      <div className="scale-menu">
        <div className="scale-menu-header">Scale References</div>
        <div>
          {scaleReferences.map((ref) => {
            const Icon = iconMap[ref.icon] || Plant;
            const isChecked = activeScales.has(ref.id);

            return (
              <div
                key={ref.id}
                className="scale-menu-item"
                onClick={(e) => {
                  e.stopPropagation();
                  onToggleScale(ref.id);
                }}
              >
                <div className={`scale-checkbox ${isChecked ? "checked" : ""}`} />
                <Icon size={18} weight="regular" />
                <div className="scale-menu-item-info">
                  <div className="scale-menu-item-name">{ref.name}</div>
                  <div className="scale-menu-item-desc">{ref.desc}</div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
