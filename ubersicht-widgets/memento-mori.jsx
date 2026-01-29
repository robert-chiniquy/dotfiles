// Memento Mori - Heartbeats since boot
// Heart rate estimated from time of day and shell activity

export const refreshFrequency = 60000; // Check every minute

const PRIME = 29; // Appears when minute % 29 === 0

// Get boot time, current hour, and recent shell activity (history line count changes)
export const command = `
  BOOT=$(sysctl -n kern.boottime | awk '{print $4}' | tr -d ',')
  HOUR=$(date +%H)
  # Count recent shell history modifications as activity proxy
  HIST_MOD=$(stat -f %m ~/.zsh_history 2>/dev/null || echo 0)
  NOW=$(date +%s)
  ACTIVITY=$(( ($NOW - $HIST_MOD) < 300 ? 1 : 0 ))
  echo "$(date +%M)|$BOOT|$HOUR|$ACTIVITY"
`;

export const render = ({ output }) => {
  if (!output) return null;
  const [minuteStr, bootStr, hourStr, activityStr] = output.trim().split('|');
  const minute = parseInt(minuteStr);

  // Visible for 3 minutes each cycle
  if (minute % PRIME >= 3) return null;

  const bootTime = parseInt(bootStr);
  const hour = parseInt(hourStr);
  const recentActivity = parseInt(activityStr);
  const now = Math.floor(Date.now() / 1000);
  const uptimeSeconds = now - bootTime;

  // Estimate heart rate based on time of day and activity
  // Base: 60 bpm resting, higher during work hours, even higher if recently active
  let bpm;
  if (hour >= 23 || hour < 6) {
    bpm = 58; // Night/sleep
  } else if (hour >= 6 && hour < 9) {
    bpm = 65; // Morning warmup
  } else if (hour >= 9 && hour < 12) {
    bpm = 72; // Morning work
  } else if (hour >= 12 && hour < 14) {
    bpm = 68; // Lunch
  } else if (hour >= 14 && hour < 18) {
    bpm = 75; // Afternoon focus
  } else if (hour >= 18 && hour < 21) {
    bpm = 70; // Evening wind down
  } else {
    bpm = 62; // Late evening
  }

  // Boost if recently active in shell
  if (recentActivity) {
    bpm += 8;
  }

  // Calculate heartbeats
  const heartbeats = Math.floor((uptimeSeconds / 60) * bpm);
  const heartbeatsFormatted = heartbeats.toLocaleString();

  // Color based on how long running
  let color;
  const uptimeHours = uptimeSeconds / 3600;
  if (uptimeHours < 4) {
    color = "#5cecff";
  } else if (uptimeHours < 8) {
    color = "#fbb725";
  } else if (uptimeHours < 16) {
    color = "#aa00e8";
  } else {
    color = "#ff0099";
  }

  return (
    <div style={container}>
      <div style={{...symbolStyle, color}}>♥</div>
      <div style={beats}>{heartbeatsFormatted}</div>
      <div style={bpmStyle}>{bpm} bpm</div>
      <div style={msg}>heartbeats</div>
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

const beats = {
  fontSize: "28px",
  fontWeight: "bold",
  color: "#ffffff",
  fontFamily: "SF Mono, monospace"
};

const bpmStyle = {
  fontSize: "14px",
  color: "#666",
  marginTop: "4px",
  fontFamily: "SF Mono, monospace"
};

const msg = {
  fontSize: "12px",
  color: "#555",
  textTransform: "uppercase",
  letterSpacing: "2px",
  marginTop: "5px",
  fontFamily: "Bradley Hand, cursive"
};
