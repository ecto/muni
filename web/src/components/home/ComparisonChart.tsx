interface ComparisonItem {
  label: string;
  value: number;
  displayValue: string;
  highlight?: boolean;
}

interface ComparisonChartProps {
  items: ComparisonItem[];
  maxValue?: number;
}

export function ComparisonChart({ items, maxValue }: ComparisonChartProps) {
  const max = maxValue ?? Math.max(...items.map((i) => i.value));

  return (
    <div className="comparison-chart">
      {items.map((item) => (
        <div
          key={item.label}
          className={`comparison-row ${item.highlight ? "comparison-row-highlight" : ""}`}
        >
          <span className="comparison-label">{item.label}</span>
          <div className="comparison-bar-container">
            <div
              className="comparison-bar"
              style={{ width: `${(item.value / max) * 100}%` }}
            />
          </div>
          <span className="comparison-value">{item.displayValue}</span>
        </div>
      ))}
    </div>
  );
}
