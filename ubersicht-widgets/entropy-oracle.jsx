// Entropy Oracle - System entropy as divination
// Higher entropy = more chaotic readings


export const refreshFrequency = 60000;

const PRIME = 41; // Appears when minute % 41 === 0

export const command = `
  # Gather entropy sources
  PROCS=$(ps aux | wc -l)
  LOAD=$(sysctl -n vm.loadavg | awk '{print int($2 * 100)}')
  RAND=$(od -An -N2 -i /dev/urandom | tr -d ' ')
  NETS=$(netstat -an 2>/dev/null | wc -l)
  echo "$(date +%M)|$PROCS|$LOAD|$RAND|$NETS"
`;

const OMENS = [
  { threshold: 0, symbol: "☉", meaning: "Sol Invictus", color: "#fbb725" },
  { threshold: 100, symbol: "☽", meaning: "Luna Descendens", color: "#5cecff" },
  { threshold: 200, symbol: "☿", meaning: "Mercurius Volatilis", color: "#aa00e8" },
  { threshold: 300, symbol: "♀", meaning: "Venus Infernus", color: "#ff00f8" },
  { threshold: 400, symbol: "♂", meaning: "Mars Ascendant", color: "#ff0099" },
  { threshold: 500, symbol: "♃", meaning: "Jupiter Regnans", color: "#5cecff" },
  { threshold: 600, symbol: "♄", meaning: "Saturnus Bound", color: "#666666" },
  { threshold: 700, symbol: "⛢", meaning: "Uranus Awakened", color: "#5cecff" },
  { threshold: 800, symbol: "♆", meaning: "Neptune Depths", color: "#aa00e8" },
  { threshold: 900, symbol: "⯓", meaning: "Pluto Rising", color: "#ff0099" },
];

export const render = ({ output }) => {
  if (!output) return null;
  const parts = output.trim().split('|').map(Number);
  const minute = parts[0];

  // Visible for 3 minutes each cycle
  if (minute % PRIME >= 3) return null;

  const [, procs, load, rand, nets] = parts;
  const entropy = (procs + load + (rand % 1000) + nets) % 1000;

  let omen = OMENS[0];
  for (const o of OMENS) {
    if (entropy >= o.threshold) omen = o;
  }

  return (
    <div style={container}>
      <div style={title}>ENTROPY ORACLE</div>
      <div style={{...symbol, color: omen.color}}>{omen.symbol}</div>
      <div style={meaning}>{omen.meaning}</div>
      <div style={value}>{entropy}</div>
    </div>
  );
};

const container = {
  position: "fixed",
  top: "320px",
  right: "60px",
  padding: "34px",
  backgroundColor: "rgba(26, 26, 46, 0.45)",
  borderRadius: "10px",
  border: "1px solid #aa00e8",
  fontFamily: "SF Pro, sans-serif",
  color: "#ffffff",
  textAlign: "center",
  backdropFilter: "blur(10px)",
  minWidth: "140px"
};

const title = {
  fontSize: "28px",
  color: "#555",
  letterSpacing: "2px",
  marginBottom: "8px"
};

const symbol = {
  fontSize: "64px",
  marginBottom: "5px"
};

const meaning = {
  fontSize: "32px",
  color: "#888",
  fontStyle: "italic",
  fontFamily: "Bradley Hand, cursive"
};

const value = {
  fontSize: "27px",
  color: "#444",
  marginTop: "8px",
  fontFamily: "SF Mono, monospace"  // Keep monospace for numbers
};
