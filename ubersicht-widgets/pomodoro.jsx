// Pomodoro Timer - Alchemical work phases
// Click to start/pause, tracks 25min work / 5min break cycles

const WORK_MINUTES = 25;
const BREAK_MINUTES = 5;

// Alchemical phases for each quarter of work session
const PHASES = [
  { name: "Nigredo", symbol: "🜔", color: "#333333", desc: "Dissolution" },
  { name: "Albedo", symbol: "🜍", color: "#ffffff", desc: "Purification" },
  { name: "Citrinitas", symbol: "🜛", color: "#fbb725", desc: "Awakening" },
  { name: "Rubedo", symbol: "🜎", color: "#ff0099", desc: "Completion" }
];


export const refreshFrequency = 1000;
export const clickThrough = false;

export const command = "cat /tmp/pomodoro-state 2>/dev/null || echo 'idle|0|0'";

export const render = ({ output }) => {
  if (!output) return null;
  const [state, startTime, totalSeconds] = output.trim().split('|');

  let display, phase, progress, stateLabel;

  if (state === 'idle') {
    display = "25:00";
    phase = { name: "Prima Materia", symbol: "☿", color: "#5cecff", desc: "Click to begin" };
    progress = 0;
    stateLabel = "READY";
  } else if (state === 'work') {
    const elapsed = Math.floor(Date.now() / 1000) - parseInt(startTime);
    const remaining = (WORK_MINUTES * 60) - elapsed;

    if (remaining <= 0) {
      display = "00:00";
      phase = PHASES[3];
      progress = 100;
      stateLabel = "COMPLETE";
    } else {
      const mins = Math.floor(remaining / 60);
      const secs = remaining % 60;
      display = `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
      progress = ((WORK_MINUTES * 60 - remaining) / (WORK_MINUTES * 60)) * 100;
      const phaseIndex = Math.min(3, Math.floor(progress / 25));
      phase = PHASES[phaseIndex];
      stateLabel = "OPUS";
    }
  } else if (state === 'break') {
    const elapsed = Math.floor(Date.now() / 1000) - parseInt(startTime);
    const remaining = (BREAK_MINUTES * 60) - elapsed;

    if (remaining <= 0) {
      display = "00:00";
      phase = { name: "Rebirth", symbol: "☉", color: "#fbb725", desc: "Begin anew" };
      progress = 100;
      stateLabel = "REST DONE";
    } else {
      const mins = Math.floor(remaining / 60);
      const secs = remaining % 60;
      display = `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
      progress = ((BREAK_MINUTES * 60 - remaining) / (BREAK_MINUTES * 60)) * 100;
      phase = { name: "Recuperatio", symbol: "☽", color: "#aa00e8", desc: "Rest" };
      stateLabel = "REST";
    }
  } else if (state === 'paused') {
    display = "--:--";
    phase = { name: "Stasis", symbol: "⏸", color: "#666666", desc: "Paused" };
    progress = parseInt(totalSeconds) || 0;
    stateLabel = "PAUSED";
  }

  const now = Math.floor(Date.now() / 1000);
  const url = (state === 'idle' || state === 'paused')
    ? `hammerspoon://pomodoro?action=start&time=${now}`
    : `hammerspoon://pomodoro?action=pause`;

  return (
    <a href={url} style={{...container, textDecoration: 'none', display: 'block'}}>
      <div style={label}>{stateLabel}</div>
      <div style={{...symbol, color: phase.color}}>{phase.symbol}</div>
      <div style={timer}>{display}</div>
      <div style={phaseName}>{phase.name}</div>
      <div style={progressBar}>
        <div style={{...progressFill, width: `${progress}%`, backgroundColor: phase.color}}></div>
      </div>
      <div style={desc}>{phase.desc}</div>
    </a>
  );
};

const container = {
  position: "fixed",
  bottom: "60px",
  left: "40px",
  padding: "24px",
  backgroundColor: "rgba(26, 26, 46, 0.45)",
  borderRadius: "14px",
  border: "1px solid #aa00e8",
  fontFamily: "SF Pro, sans-serif",
  color: "#ffffff",
  textAlign: "center",
  backdropFilter: "blur(10px)",
  cursor: "pointer",
  minWidth: "160px"
};

const label = {
  fontSize: "10px",
  color: "#555",
  letterSpacing: "3px",
  marginBottom: "8px"
};

const symbol = {
  fontSize: "27px",
  marginBottom: "8px"
};

const timer = {
  fontSize: "27px",
  fontWeight: "bold",
  fontFamily: "SF Mono, monospace",
  color: "#ffffff",
  marginBottom: "4px"
};

const phaseName = {
  fontSize: "14px",
  color: "#ff0099",
  fontStyle: "italic",
  marginBottom: "12px"
};

const progressBar = {
  width: "100%",
  height: "4px",
  backgroundColor: "rgba(255,255,255,0.1)",
  borderRadius: "2px",
  overflow: "hidden",
  marginBottom: "8px"
};

const progressFill = {
  height: "100%",
  transition: "width 1s linear"
};

const desc = {
  fontSize: "11px",
  color: "#666"
};
