// Git Sigil - Current repo state as occult symbol
// Changes based on git status of current directory


export const refreshFrequency = 60000;
export const clickThrough = false;

const PRIME = 43; // Appears when minute % 43 === 0

export const command = `
  cd ~/repo 2>/dev/null || cd ~
  REPO_PATH=$(pwd)
  if git rev-parse --git-dir > /dev/null 2>&1; then
    DIRTY=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
    BRANCH=$(git branch --show-current 2>/dev/null)
    AHEAD=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo 0)
    BEHIND=$(git rev-list --count HEAD..@{u} 2>/dev/null || echo 0)
    echo "$(date +%M)|$DIRTY|$BRANCH|$AHEAD|$BEHIND|$REPO_PATH"
  else
    echo "$(date +%M)|none||||| "
  fi
`;

export const render = ({ output }) => {
  if (!output) return null;
  const parts = output.trim().split("|");
  const minute = parseInt(parts[0]);

  // Visible for 4 minutes each cycle
  if (minute % PRIME >= 4) return null;

  if (parts[1] === "none") {
    return <div style={container}><div style={symbolStyle}>◌</div><div style={label}>The Void</div></div>;
  }

  const [, dirty, branch, ahead, behind, repoPath] = parts;
  const dirtyNum = parseInt(dirty) || 0;
  const aheadNum = parseInt(ahead) || 0;
  const behindNum = parseInt(behind) || 0;

  let sigil, color, state;

  if (dirtyNum === 0 && aheadNum === 0 && behindNum === 0) {
    sigil = "⬡"; // Hexagon - perfect harmony
    color = "#5cecff";
    state = "Pure";
  } else if (dirtyNum > 0 && aheadNum === 0) {
    sigil = "🜂"; // Fire - work in progress
    color = "#fbb725";
    state = `${dirtyNum} unclean`;
  } else if (aheadNum > 0 && dirtyNum === 0) {
    sigil = "🜁"; // Air - ready to release
    color = "#aa00e8";
    state = `${aheadNum} to push`;
  } else if (behindNum > 0) {
    sigil = "🜄"; // Water - need to pull
    color = "#ff0099";
    state = `${behindNum} behind`;
  } else {
    sigil = "🜃"; // Earth - mixed state
    color = "#fbb725";
    state = "In flux";
  }

  // Click to open repo in terminal
  const openUrl = `hammerspoon://git?action=open&path=${encodeURIComponent(repoPath)}`;

  return (
    <a href={openUrl} style={{...container, textDecoration: 'none', display: 'block'}}>
      <div style={{...symbolStyle, color}}>{sigil}</div>
      <div style={branchStyle}>{branch}</div>
      <div style={label}>{state}</div>
    </a>
  );
};

const container = {
  position: "fixed",
  top: "180px",
  right: "160px",
  padding: "34px",
  backgroundColor: "rgba(26, 26, 46, 0.45)",
  borderRadius: "10px",
  border: "1px solid #333",
  fontFamily: "SF Pro, Helvetica Neue, sans-serif",
  color: "#ffffff",
  textAlign: "center",
  backdropFilter: "blur(10px)",
  minWidth: "80px"
};

const symbolStyle = {
  fontSize: "64px",
  marginBottom: "5px"
};

const branchStyle = {
  fontSize: "28px",
  color: "#5cecff",
  fontFamily: "Bradley Hand, cursive"
};

const label = {
  fontSize: "32px",
  color: "#666",
  marginTop: "5px",
  fontFamily: "Bradley Hand, cursive"
};
