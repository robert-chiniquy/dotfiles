// Network Ley Lines - Active connections as mystical pathways
// Each connection type gets a different symbol


export const refreshFrequency = 60000;

const PRIME = 53; // Appears when minute % 53 === 0

export const command = `
  echo "$(date +%M)|$(netstat -an 2>/dev/null | grep ESTABLISHED | wc -l | tr -d ' ')|$(netstat -an 2>/dev/null | grep LISTEN | wc -l | tr -d ' ')|$(netstat -an 2>/dev/null | grep TIME_WAIT | wc -l | tr -d ' ')|$(route -n get default 2>/dev/null | grep gateway | awk '{print $2}' | cut -d. -f1-2)"
`;

export const render = ({ output }) => {
  if (!output) return null;
  const parts = output.trim().split('|');
  const minute = parseInt(parts[0]);

  // Visible for 4 minutes each cycle
  if (minute % PRIME >= 4) return null;

  const established = parseInt(parts[1]) || 0;
  const listening = parseInt(parts[2]) || 0;
  const waiting = parseInt(parts[3]) || 0;
  const gateway = parts[4] || "void";

  return (
    <div style={container}>
      <div style={title}>LEY LINES</div>
      <div style={grid}>
        <div style={line}>
          <span style={{...symbol, color: "#5cecff"}}>⚡</span>
          <span style={label}>Active</span>
          <span style={num}>{established}</span>
        </div>
        <div style={line}>
          <span style={{...symbol, color: "#aa00e8"}}>👁</span>
          <span style={label}>Listening</span>
          <span style={num}>{listening}</span>
        </div>
        <div style={line}>
          <span style={{...symbol, color: "#666"}}>⌛</span>
          <span style={label}>Liminal</span>
          <span style={num}>{waiting}</span>
        </div>
      </div>
      <div style={gatewayStyle}>Gateway: {gateway}.*</div>
    </div>
  );
};

const container = {
  position: "fixed",
  top: "460px",
  right: "50px",
  padding: "34px",
  backgroundColor: "rgba(26, 26, 46, 0.45)",
  borderRadius: "10px",
  border: "1px solid #333",
  fontFamily: "SF Pro, sans-serif",
  color: "#ffffff",
  backdropFilter: "blur(10px)",
  minWidth: "160px"
};

const title = {
  fontSize: "28px",
  color: "#555",
  letterSpacing: "2px",
  marginBottom: "10px",
  textAlign: "center"
};

const grid = {
  display: "flex",
  flexDirection: "column",
  gap: "6px"
};

const line = {
  display: "flex",
  alignItems: "center",
  fontSize: "28px"
};

const symbol = {
  fontSize: "32px",
  marginRight: "16px",
  width: "24px"
};

const label = {
  flex: 1,
  color: "#666",
  fontSize: "32px",
  fontFamily: "Bradley Hand, cursive"
};

const num = {
  color: "#fff",
  fontFamily: "SF Mono, monospace",
  fontSize: "28px"
};

const gatewayStyle = {
  marginTop: "10px",
  fontSize: "27px",
  color: "#444",
  fontFamily: "SF Mono, monospace",
  textAlign: "center"
};
