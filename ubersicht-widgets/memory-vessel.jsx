// Memory Vessel - RAM as alchemical container
// Pressure shown as liquid level with mystical states


export const refreshFrequency = 60000;

const PRIME = 47; // Appears when minute % 47 === 0

export const command = `
  echo "$(date +%M)|$(vm_stat | grep -E 'Pages (free|active|inactive|wired)' | awk '{print $NF}' | tr -d '.' | paste -sd'|' -)"
`;

export const render = ({ output }) => {
  if (!output) return null;
  const parts = output.trim().split('|').map(Number);
  const minute = parts[0];

  // Visible for 3 minutes each cycle
  if (minute % PRIME >= 3) return null;

  const pageSize = 16384; // 16KB pages on Apple Silicon
  const [, free, active, inactive, wired] = parts;

  const totalPages = free + active + inactive + wired;
  const usedPercent = Math.round(((active + wired) / totalPages) * 100);

  // Alchemical state based on memory pressure
  let state, stateColor, symbol;
  if (usedPercent < 40) {
    state = "Aqua Vitae";
    stateColor = "#5cecff";
    symbol = "🜄";
  } else if (usedPercent < 60) {
    state = "Spiritus";
    stateColor = "#fbb725";
    symbol = "🜁";
  } else if (usedPercent < 80) {
    state = "Ignis";
    stateColor = "#ff0099";
    symbol = "🜂";
  } else {
    state = "Terra Firma";
    stateColor = "#aa00e8";
    symbol = "🜃";
  }

  const usedGB = ((active + wired) * pageSize / 1073741824).toFixed(1);
  const totalGB = (totalPages * pageSize / 1073741824).toFixed(0);

  return (
    <div style={container}>
      <div style={title}>MEMORY VESSEL</div>
      <div style={vessel}>
        <div style={{...liquid, height: `${usedPercent}%`, backgroundColor: stateColor}}></div>
        <div style={vesselSymbol}>{symbol}</div>
      </div>
      <div style={{...stateText, color: stateColor}}>{state}</div>
      <div style={stats}>{usedGB}G / {totalGB}G</div>
    </div>
  );
};

const container = {
  position: "fixed",
  top: "600px",
  right: "60px",
  padding: "34px",
  backgroundColor: "rgba(26, 26, 46, 0.45)",
  borderRadius: "10px",
  border: "1px solid #333",
  fontFamily: "SF Pro, sans-serif",
  color: "#ffffff",
  textAlign: "center",
  backdropFilter: "blur(10px)"
};

const title = {
  fontSize: "28px",
  color: "#555",
  letterSpacing: "2px",
  marginBottom: "10px"
};

const vessel = {
  width: "50px",
  height: "70px",
  margin: "0 auto",
  border: "2px solid #444",
  borderRadius: "0 0 25px 25px",
  position: "relative",
  overflow: "hidden",
  backgroundColor: "rgba(0,0,0,0.3)"
};

const liquid = {
  position: "absolute",
  bottom: 0,
  left: 0,
  right: 0,
  opacity: 0.7,
  transition: "height 1s ease"
};

const vesselSymbol = {
  position: "absolute",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
  fontSize: "27px",
  opacity: 0.8
};

const stateText = {
  fontSize: "32px",
  marginTop: "8px",
  fontStyle: "italic",
  fontFamily: "Bradley Hand, cursive"
};

const stats = {
  fontSize: "27px",
  color: "#555",
  marginTop: "4px",
  fontFamily: "SF Mono, monospace"
};
