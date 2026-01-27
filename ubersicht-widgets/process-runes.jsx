// Process Runes - Top processes as mystical glyphs
// Each process gets a rune based on its name hash

const RUNES = "ᚠᚢᚦᚨᚱᚲᚷᚹᚺᚾᛁᛃᛇᛈᛉᛊᛏᛒᛖᛗᛚᛜᛞᛟ";


export const refreshFrequency = 60000;

const PRIME = 59; // Appears when minute % 59 === 0

export const command = "echo \"$(date +%M)\"; ps -Arco '%cpu,comm' | head -6 | tail -5";

export const render = ({ output }) => {
  if (!output) return null;
  const allLines = output.trim().split('\n');
  const minute = parseInt(allLines[0]);

  // Visible for 3 minutes each cycle
  if (minute % PRIME >= 3) return null;

  const lines = allLines.slice(1).filter(l => l.trim());

  const processes = lines.map(line => {
    const match = line.trim().match(/^\s*([\d.]+)\s+(.+)$/);
    if (!match) return null;
    const [, cpu, name] = match;
    const hash = name.split('').reduce((a, c) => a + c.charCodeAt(0), 0);
    const rune = RUNES[hash % RUNES.length];
    const cpuNum = parseFloat(cpu);
    return { name: name.slice(0, 12), cpu: cpuNum, rune };
  }).filter(Boolean);

  return (
    <div style={container}>
      <div style={title}>SERVITORS</div>
      {processes.map((p, i) => (
        <div key={i} style={row}>
          <span style={{...rune, opacity: Math.min(1, 0.3 + p.cpu/50)}}>{p.rune}</span>
          <span style={name}>{p.name}</span>
          <span style={cpu}>{p.cpu.toFixed(0)}%</span>
        </div>
      ))}
    </div>
  );
};

const container = {
  position: "fixed",
  top: "70px",
  left: "10px",
  padding: "34px",
  backgroundColor: "rgba(26, 26, 46, 0.45)",
  borderRadius: "10px",
  border: "1px solid #333",
  fontFamily: "SF Pro, sans-serif",
  color: "#ffffff",
  backdropFilter: "blur(10px)",
  minWidth: "200px"
};

const title = {
  fontSize: "32px",
  color: "#666",
  letterSpacing: "2px",
  marginBottom: "10px",
  textAlign: "center"
};

const row = {
  display: "flex",
  alignItems: "center",
  marginBottom: "6px",
  fontSize: "27px"
};

const rune = {
  fontSize: "27px",
  color: "#ff0099",
  marginRight: "16px",
  width: "24px"
};

const name = {
  flex: 1,
  color: "#888",
  fontFamily: "SF Mono, monospace",
  fontSize: "32px"
};

const cpu = {
  color: "#5cecff",
  fontFamily: "SF Mono, monospace",
  fontSize: "32px"
};
