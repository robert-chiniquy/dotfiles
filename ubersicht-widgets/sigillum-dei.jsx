// Sigillum Dei - John Dee's seal rendered from system state
// The seven planetary seals based on current metrics


export const refreshFrequency = 60000; // Check every minute

const PRIME = 23; // Appears when minute % 23 === 0

export const command = `
  echo "$(date +%M)"                                    # Minute for prime check
  echo "|$(date +%H)"                                   # Hour (Sol position)
  echo "|$(sysctl -n vm.loadavg | awk '{print int($2*10)}')"  # Load (activity)
  echo "|$(ps aux | wc -l | tr -d ' ')"                # Processes
  echo "|$(netstat -an 2>/dev/null | grep EST | wc -l | tr -d ' ')"  # Connections
  echo "|$(df / | tail -1 | awk '{print int($5)}')"   # Disk %
  echo "|$(vm_stat | grep 'Pages active' | awk '{print int($3/1000)}')"  # Memory
  echo "|$(date +%u)"                                  # Day of week
`;

const SEALS = [
  { planet: "Sol", symbol: "☉", angle: 0 },
  { planet: "Luna", symbol: "☽", angle: 51.4 },
  { planet: "Mars", symbol: "♂", angle: 102.8 },
  { planet: "Mercury", symbol: "☿", angle: 154.3 },
  { planet: "Jupiter", symbol: "♃", angle: 205.7 },
  { planet: "Venus", symbol: "♀", angle: 257.1 },
  { planet: "Saturn", symbol: "♄", angle: 308.6 }
];

export const render = ({ output }) => {
  if (!output) return null;
  const values = output.replace(/\n/g, '').split('|').map(Number);
  const minute = values[0];

  // Visible for 4 minutes each cycle
  if (minute % PRIME >= 4) return null;

  const hour = values[1];

  // Determine active seal based on hour
  const activeSeal = hour % 7;

  return (
    <div style={container}>
      <div style={title}>SIGILLUM DEI</div>
      <div style={circle}>
        {SEALS.map((seal, i) => {
          const isActive = i === activeSeal;
          const rad = (seal.angle - 90) * Math.PI / 180;
          const x = 35 + 28 * Math.cos(rad);
          const y = 35 + 28 * Math.sin(rad);
          return (
            <div key={i} style={{
              ...sealSymbol,
              left: `${x}px`,
              top: `${y}px`,
              color: isActive ? "#ff0099" : "#444",
              fontSize: isActive ? "16px" : "12px",
              textShadow: isActive ? "0 0 10px #ff0099" : "none"
            }}>{seal.symbol}</div>
          );
        })}
        <div style={centerSymbol}>✡</div>
      </div>
      <div style={activeName}>{SEALS[activeSeal].planet}</div>
    </div>
  );
};

const container = {
  position: "fixed",
  top: "520px",
  left: "40px",
  padding: "34px",
  backgroundColor: "rgba(26, 26, 46, 0.45)",
  borderRadius: "10px",
  border: "1px solid #aa00e8",
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

const circle = {
  width: "70px",
  height: "70px",
  margin: "0 auto",
  border: "1px solid #333",
  borderRadius: "50%",
  position: "relative"
};

const sealSymbol = {
  position: "absolute",
  transform: "translate(-50%, -50%)",
  transition: "all 0.5s ease"
};

const centerSymbol = {
  position: "absolute",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
  fontSize: "32px",
  color: "#aa00e8"
};

const activeName = {
  marginTop: "8px",
  fontSize: "32px",
  color: "#ff0099",
  fontStyle: "italic"
};
