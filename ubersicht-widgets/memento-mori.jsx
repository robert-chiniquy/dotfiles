// Memento Mori - Uptime as mortality reminder
// The longer you've been working, the more ominous it gets


export const refreshFrequency = 60000; // Check every minute

const PRIME = 29; // Appears when minute % 29 === 0

export const command = "echo \"$(date +%M)|$(sysctl -n kern.boottime | awk '{print $4}' | tr -d ',')\"";

export const render = ({ output }) => {
  if (!output) return null;
  const [minuteStr, bootStr] = output.trim().split('|');
  const minute = parseInt(minuteStr);

  // Visible for 3 minutes each cycle
  if (minute % PRIME >= 3) return null;

  const bootTime = parseInt(bootStr);
  const now = Math.floor(Date.now() / 1000);
  const uptimeHours = Math.floor((now - bootTime) / 3600);

  let symbol, message, color;

  if (uptimeHours < 2) {
    symbol = "⏳";
    message = "Time flows";
    color = "#5cecff";
  } else if (uptimeHours < 6) {
    symbol = "⌛";
    message = "Hours pass";
    color = "#fbb725";
  } else if (uptimeHours < 12) {
    symbol = "☠";
    message = "Rest soon";
    color = "#aa00e8";
  } else if (uptimeHours < 24) {
    symbol = "⚰";
    message = "Mortal coil";
    color = "#ff0099";
  } else {
    symbol = "💀";
    message = "Memento mori";
    color = "#ff0099";
  }

  return (
    <div style={container}>
      <div style={{...symbolStyle, color}}>{symbol}</div>
      <div style={hours}>{uptimeHours}h</div>
      <div style={msg}>{message}</div>
    </div>
  );
};

const container = {
  position: "fixed",
  top: "60px",
  right: "60px",
  padding: "34px",
  backgroundColor: "rgba(26, 26, 46, 0.45)",
  borderRadius: "10px",
  border: "1px solid #333",
  fontFamily: "SF Pro, Helvetica Neue, sans-serif",
  color: "#ffffff",
  textAlign: "center",
  backdropFilter: "blur(10px)"
};

const symbolStyle = {
  fontSize: "32px",
  marginBottom: "5px"
};

const hours = {
  fontSize: "32px",
  fontWeight: "bold",
  color: "#ffffff"
};

const msg = {
  fontSize: "32px",
  color: "#666",
  textTransform: "uppercase",
  letterSpacing: "2px",
  marginTop: "5px"
};
