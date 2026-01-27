// Daily Tarot Card - Major Arcana

const MAJOR_ARCANA = [
  { num: 0, name: "The Fool", symbol: "🃏", keywords: ["Beginnings", "Innocence", "Spontaneity"], upright: "New beginnings, innocence, spontaneity, a free spirit.", shadow: "Recklessness, risk-taking." },
  { num: 1, name: "The Magician", symbol: "✧", keywords: ["Manifestation", "Power", "Action"], upright: "Manifestation, resourcefulness, power, inspired action.", shadow: "Manipulation, poor planning." },
  { num: 2, name: "High Priestess", symbol: "☽", keywords: ["Intuition", "Mystery", "Subconscious"], upright: "Intuition, sacred knowledge, divine feminine.", shadow: "Secrets, withdrawal." },
  { num: 3, name: "The Empress", symbol: "♀", keywords: ["Fertility", "Beauty", "Nature"], upright: "Femininity, beauty, nature, abundance.", shadow: "Creative block, dependence." },
  { num: 4, name: "The Emperor", symbol: "♔", keywords: ["Authority", "Structure", "Control"], upright: "Authority, structure, a father figure.", shadow: "Domination, rigidity." },
  { num: 5, name: "Hierophant", symbol: "⚜", keywords: ["Tradition", "Conformity", "Morality"], upright: "Spiritual wisdom, tradition, conformity.", shadow: "Challenging status quo." },
  { num: 6, name: "The Lovers", symbol: "❤", keywords: ["Love", "Harmony", "Choices"], upright: "Love, harmony, relationships, choices.", shadow: "Disharmony, imbalance." },
  { num: 7, name: "The Chariot", symbol: "⚔", keywords: ["Control", "Willpower", "Success"], upright: "Control, willpower, determination.", shadow: "Lack of direction." },
  { num: 8, name: "Strength", symbol: "∞", keywords: ["Courage", "Patience", "Compassion"], upright: "Strength, courage, compassion.", shadow: "Self-doubt, weakness." },
  { num: 9, name: "The Hermit", symbol: "☆", keywords: ["Solitude", "Guidance", "Introspection"], upright: "Soul-searching, inner guidance.", shadow: "Isolation, loneliness." },
  { num: 10, name: "Wheel", symbol: "☸", keywords: ["Change", "Cycles", "Fate"], upright: "Good luck, karma, turning point.", shadow: "Bad luck, resistance." },
  { num: 11, name: "Justice", symbol: "⚖", keywords: ["Justice", "Fairness", "Truth"], upright: "Justice, fairness, truth, karma.", shadow: "Unfairness, dishonesty." },
  { num: 12, name: "Hanged Man", symbol: "⊥", keywords: ["Pause", "Surrender", "Perspective"], upright: "Pause, surrender, new perspectives.", shadow: "Delays, resistance." },
  { num: 13, name: "Death", symbol: "☠", keywords: ["Endings", "Change", "Transformation"], upright: "Endings, transformation, transition.", shadow: "Resistance to change." },
  { num: 14, name: "Temperance", symbol: "△", keywords: ["Balance", "Moderation", "Patience"], upright: "Balance, moderation, patience.", shadow: "Imbalance, excess." },
  { num: 15, name: "The Devil", symbol: "⛧", keywords: ["Shadow", "Attachment", "Addiction"], upright: "Shadow self, attachment, addiction.", shadow: "Releasing limitations." },
  { num: 16, name: "The Tower", symbol: "⚡", keywords: ["Upheaval", "Chaos", "Revelation"], upright: "Sudden change, upheaval, awakening.", shadow: "Fear of change." },
  { num: 17, name: "The Star", symbol: "✦", keywords: ["Hope", "Faith", "Renewal"], upright: "Hope, faith, renewal, serenity.", shadow: "Lack of faith, despair." },
  { num: 18, name: "The Moon", symbol: "☾", keywords: ["Illusion", "Fear", "Intuition"], upright: "Illusion, fear, subconscious.", shadow: "Release of fear." },
  { num: 19, name: "The Sun", symbol: "☀", keywords: ["Joy", "Success", "Vitality"], upright: "Positivity, warmth, success.", shadow: "Inner child, sadness." },
  { num: 20, name: "Judgement", symbol: "⚱", keywords: ["Rebirth", "Reflection", "Reckoning"], upright: "Judgement, rebirth, inner calling.", shadow: "Self-doubt, ignoring call." },
  { num: 21, name: "The World", symbol: "◯", keywords: ["Completion", "Integration", "Achievement"], upright: "Completion, accomplishment, travel.", shadow: "Seeking closure." }
];


export const refreshFrequency = 60000;
export const command = "date '+%Y%m%d %M'";

const PRIME = 31; // Appears when minute % 31 === 0

const handleClick = () => {
  window.location.href = "https://en.wikipedia.org/wiki/Major_Arcana";
};

function toRoman(num) {
  const map = [[21,"XXI"],[20,"XX"],[19,"XIX"],[18,"XVIII"],[17,"XVII"],[16,"XVI"],[15,"XV"],[14,"XIV"],[13,"XIII"],[12,"XII"],[11,"XI"],[10,"X"],[9,"IX"],[8,"VIII"],[7,"VII"],[6,"VI"],[5,"V"],[4,"IV"],[3,"III"],[2,"II"],[1,"I"]];
  for (const [value, roman] of map) if (num >= value) return roman;
  return "0";
}

export const render = ({ output }) => {
  if (!output) return null;
  const [dateStr, minuteStr] = output.trim().split(' ');
  const minute = parseInt(minuteStr);

  // Visible for 10 minutes each cycle
  if (minute % PRIME >= 10) return null;

  const day = parseInt(dateStr);
  const idx = day % 22;
  const card = MAJOR_ARCANA[idx];

  return (
    <div style={container} onClick={handleClick}>
      <div style={cardFrame}>
        <div style={numeral}>{card.num === 0 ? "0" : toRoman(card.num)}</div>
        <div style={symbol}>{card.symbol}</div>
        <div style={name}>{card.name}</div>
      </div>
      <div style={keywords}>
        {card.keywords.map((k, i) => (<span key={i} style={keyword}>{k}</span>))}
      </div>
      <div style={meaning}>{card.upright}</div>
      <div style={shadow}>Shadow: {card.shadow}</div>
    </div>
  );
};

const container = {
  position: "fixed",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
  width: "400px",
  padding: "32px",
  backgroundColor: "rgba(26, 26, 46, 0.45)",
  borderRadius: "14px",
  border: "2px solid #ff0099",
  fontFamily: "SF Pro, Helvetica Neue, sans-serif",
  color: "#ffffff",
  backdropFilter: "blur(10px)",
  cursor: "pointer"
};

const cardFrame = {
  textAlign: "center",
  padding: "18px",
  marginBottom: "18px",
  border: "1px solid #444",
  borderRadius: "10px",
  background: "linear-gradient(180deg, rgba(255,0,153,0.1) 0%, rgba(170,0,232,0.1) 100%)"
};

const numeral = {
  fontSize: "27px",
  color: "#666",
  letterSpacing: "3px",
  marginBottom: "12px"
};

const symbol = {
  fontSize: "72px",
  color: "#fbb725",
  margin: "12px 0"
};

const name = {
  fontSize: "28px",
  fontWeight: "bold",
  color: "#ff0099",
  textTransform: "uppercase",
  letterSpacing: "2px"
};

const keywords = {
  display: "flex",
  flexWrap: "wrap",
  gap: "8px",
  marginBottom: "14px",
  justifyContent: "center"
};

const keyword = {
  fontSize: "28px",
  padding: "4px 10px",
  backgroundColor: "rgba(92, 236, 255, 0.2)",
  color: "#5cecff",
  borderRadius: "12px",
  textTransform: "uppercase",
  letterSpacing: "1px"
};

const meaning = {
  fontSize: "30px",
  lineHeight: "1.6",
  color: "#ffffff",
  marginBottom: "12px"
};

const shadow = {
  fontSize: "27px",
  color: "#666",
  fontStyle: "italic",
  borderTop: "1px solid #333",
  paddingTop: "12px"
};
