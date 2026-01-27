// Disk Grimoire - Storage as pages in a spellbook
// Each major directory is a chapter


export const refreshFrequency = 60000; // 1 minute
export const clickThrough = false;

const PRIME = 37; // Appears when minute % 37 === 0

export const command = `
  DISK=$(df -h / | tail -1 | awk '{print $3 "|" $4 "|" $5}')
  # Find largest subdirectory in ~/repo
  LARGEST=$(du -sh ~/repo/* 2>/dev/null | sort -hr | head -1 | awk '{print $2}')
  echo "$(date +%M)|$DISK|$LARGEST"
`;

export const render = ({ output }) => {
  if (!output) return null;
  const parts = output.trim().split('|');
  const minute = parseInt(parts[0]);

  // Visible for 4 minutes each cycle
  if (minute % PRIME >= 4) return null;

  const [, used, avail, percent, largestDir] = parts;
  const pctNum = parseInt(percent);

  // Grimoire state
  let state, color;
  if (pctNum < 50) {
    state = "Pages Unwritten";
    color = "#5cecff";
  } else if (pctNum < 75) {
    state = "Tome Filling";
    color = "#fbb725";
  } else if (pctNum < 90) {
    state = "Chapters Dense";
    color = "#ff0099";
  } else {
    state = "Grimoire Full";
    color = "#aa00e8";
  }

  // Click to open Finder at largest directory
  const openUrl = `hammerspoon://finder?path=${encodeURIComponent(largestDir || '/')}`;

  return (
    <a href={openUrl} style={{...container, textDecoration: 'none', display: 'block'}}>
      <div style={title}>GRIMOIRE</div>
      <div style={book}>
        <div style={spine}></div>
        <div style={pages}>
          <div style={{...filled, width: `${pctNum}%`, backgroundColor: color}}></div>
        </div>
      </div>
      <div style={{...stateText, color}}>{state}</div>
      <div style={stats}>{used} inscribed</div>
      <div style={stats}>{avail} blank</div>
    </a>
  );
};

const container = {
  position: "fixed",
  top: "600px",
  left: "160px",
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

const book = {
  display: "flex",
  width: "70px",
  height: "50px",
  margin: "0 auto"
};

const spine = {
  width: "8px",
  backgroundColor: "#333",
  borderRadius: "2px 0 0 2px"
};

const pages = {
  flex: 1,
  backgroundColor: "rgba(255,255,255,0.1)",
  borderRadius: "0 2px 2px 0",
  position: "relative",
  overflow: "hidden"
};

const filled = {
  position: "absolute",
  top: 0,
  left: 0,
  bottom: 0,
  opacity: 0.6
};

const stateText = {
  fontSize: "32px",
  marginTop: "8px",
  fontStyle: "italic"
};

const stats = {
  fontSize: "27px",
  color: "#555",
  marginTop: "2px",
  fontFamily: "SF Mono, monospace"
};
