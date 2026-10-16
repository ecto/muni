import { deskGate, stages } from "@/lib/plan";

// The climb: eight stages, one scale, from a desk to a star. Only the first is a plan.
export function StageStrip() {
  return (
    <>
      <ol className="stages">
        {stages.map((s, i) => (
          <li key={s.name} className={i === 0 ? "stage is-now" : "stage"}>
            <span className="stage-num">{String(i + 1).padStart(2, "0")}</span>
            <span className="stage-name">{s.name}</span>
            <span className="stage-what">{s.what}</span>
            <span className="stage-watts">{s.watts}</span>
          </li>
        ))}
      </ol>
      <p className="stage-gate">
        <b>Desk is the plan.</b> {deskGate}
      </p>
    </>
  );
}
