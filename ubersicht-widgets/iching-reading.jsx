// I Ching Daily Reading Widget
// Displays hexagram with full interpretation

const HEXAGRAMS = [
  { num: 1, name: "Qian", title: "The Creative", hex: "䷀",
    meaning: "Pure creative force. Initiative brings success. Perseverance furthers.",
    image: "Heaven above, heaven below. The movement of heaven is full of power." },
  { num: 2, name: "Kun", title: "The Receptive", hex: "䷁",
    meaning: "Receptive devotion. The mare's perseverance brings good fortune.",
    image: "Earth above, earth below. The earth's condition is receptive devotion." },
  { num: 3, name: "Zhun", title: "Difficulty at the Beginning", hex: "䷂",
    meaning: "Initial difficulty. Clouds and thunder. Perseverance brings success.",
    image: "Water above, thunder below. Chaos gives birth to order." },
  { num: 4, name: "Meng", title: "Youthful Folly", hex: "䷃",
    meaning: "Inexperience. The young fool seeks me. I do not seek the young fool.",
    image: "Mountain above, water below. A spring wells up at the foot of the mountain." },
  { num: 5, name: "Xu", title: "Waiting", hex: "䷄",
    meaning: "Patient waiting. Sincerity leads to brilliant success.",
    image: "Water above, heaven below. Clouds rise up to heaven." },
  { num: 6, name: "Song", title: "Conflict", hex: "䷅",
    meaning: "Conflict. Seek the witch. Crossing the great water brings misfortune.",
    image: "Heaven above, water below. Heaven and water go opposite ways." },
  { num: 7, name: "Shi", title: "The Army", hex: "䷆",
    meaning: "The army requires discipline. The elder leads. Good fortune without blame.",
    image: "Earth above, water below. Water in the midst of the earth." },
  { num: 8, name: "Bi", title: "Holding Together", hex: "䷇",
    meaning: "Union brings good fortune. Those who are uncertain gradually join.",
    image: "Water above, earth below. On the earth is water." },
  { num: 9, name: "Xiao Chu", title: "Small Taming", hex: "䷈",
    meaning: "The small tames. Dense clouds, no rain from our western region.",
    image: "Wind above, heaven below. Wind drives across heaven." },
  { num: 10, name: "Lu", title: "Treading", hex: "䷉",
    meaning: "Treading upon the tiger's tail. It does not bite. Success.",
    image: "Heaven above, lake below. Heaven above, the lake below." },
  { num: 11, name: "Tai", title: "Peace", hex: "䷊",
    meaning: "Peace. The small departs, the great approaches. Good fortune. Success.",
    image: "Earth above, heaven below. Heaven and earth unite." },
  { num: 12, name: "Pi", title: "Standstill", hex: "䷋",
    meaning: "Stagnation. Evil people do not further the perseverance of the superior.",
    image: "Heaven above, earth below. Heaven and earth do not unite." },
  { num: 13, name: "Tong Ren", title: "Fellowship", hex: "䷌",
    meaning: "Fellowship in the open. Success. Crossing the great water furthers.",
    image: "Heaven above, fire below. Heaven together with fire." },
  { num: 14, name: "Da You", title: "Great Possession", hex: "䷍",
    meaning: "Great possession. Supreme success.",
    image: "Fire above, heaven below. Fire in heaven above." },
  { num: 15, name: "Qian", title: "Modesty", hex: "䷎",
    meaning: "Modesty creates success. The superior carries things through.",
    image: "Earth above, mountain below. Within earth, a mountain." },
  { num: 16, name: "Yu", title: "Enthusiasm", hex: "䷏",
    meaning: "Enthusiasm. It furthers one to appoint helpers and set armies marching.",
    image: "Thunder above, earth below. Thunder comes out of the earth." },
  { num: 17, name: "Sui", title: "Following", hex: "䷐",
    meaning: "Following brings supreme success. Perseverance furthers. No blame.",
    image: "Lake above, thunder below. Thunder in the middle of the lake." },
  { num: 18, name: "Gu", title: "Work on the Decayed", hex: "䷑",
    meaning: "Work on what has been spoiled. Crossing the great water furthers.",
    image: "Mountain above, wind below. Wind blows low on the mountain." },
  { num: 19, name: "Lin", title: "Approach", hex: "䷒",
    meaning: "Approach brings supreme success. Perseverance furthers.",
    image: "Earth above, lake below. The earth above the lake." },
  { num: 20, name: "Guan", title: "Contemplation", hex: "䷓",
    meaning: "Contemplation. Ablution, but not yet the offering. Full of trust they look up.",
    image: "Wind above, earth below. Wind blows over the earth." },
  { num: 21, name: "Shi He", title: "Biting Through", hex: "䷔",
    meaning: "Biting through brings success. It furthers to let justice be administered.",
    image: "Fire above, thunder below. Thunder and lightning." },
  { num: 22, name: "Bi", title: "Grace", hex: "䷕",
    meaning: "Grace brings success. In small matters it furthers to undertake something.",
    image: "Mountain above, fire below. Fire at the foot of the mountain." },
  { num: 23, name: "Bo", title: "Splitting Apart", hex: "䷖",
    meaning: "Splitting apart. It does not further to go anywhere.",
    image: "Mountain above, earth below. The mountain rests on the earth." },
  { num: 24, name: "Fu", title: "Return", hex: "䷗",
    meaning: "Return. Success. Going out and coming in without error.",
    image: "Earth above, thunder below. Thunder within the earth." },
  { num: 25, name: "Wu Wang", title: "Innocence", hex: "䷘",
    meaning: "Innocence. Supreme success. Perseverance furthers.",
    image: "Heaven above, thunder below. Under heaven thunder rolls." },
  { num: 26, name: "Da Chu", title: "Great Taming", hex: "䷙",
    meaning: "Great taming. Perseverance furthers. Not eating at home brings good fortune.",
    image: "Mountain above, heaven below. Heaven within the mountain." },
  { num: 27, name: "Yi", title: "Nourishment", hex: "䷚",
    meaning: "Nourishment. Perseverance brings good fortune. Pay heed to nourishment.",
    image: "Mountain above, thunder below. Thunder at the foot of the mountain." },
  { num: 28, name: "Da Guo", title: "Great Exceeding", hex: "䷛",
    meaning: "Great exceeding. The ridgepole sags. It furthers to have somewhere to go.",
    image: "Lake above, wind below. Lake over wood." },
  { num: 29, name: "Kan", title: "The Abysmal", hex: "䷜",
    meaning: "The abysmal repeated. Sincerity brings success of the heart.",
    image: "Water above, water below. Water flows on and reaches the goal." },
  { num: 30, name: "Li", title: "The Clinging", hex: "䷝",
    meaning: "The clinging. Perseverance furthers. Care of the cow brings good fortune.",
    image: "Fire above, fire below. Brightness rises twice." },
  { num: 31, name: "Xian", title: "Influence", hex: "䷞",
    meaning: "Influence. Success. Perseverance furthers. To take a maiden to wife is good.",
    image: "Lake above, mountain below. A lake on the mountain." },
  { num: 32, name: "Heng", title: "Duration", hex: "䷟",
    meaning: "Duration. Success. No blame. Perseverance furthers.",
    image: "Thunder above, wind below. Thunder and wind." },
  { num: 33, name: "Dun", title: "Retreat", hex: "䷠",
    meaning: "Retreat. Success. In small matters perseverance furthers.",
    image: "Heaven above, mountain below. Mountain under heaven." },
  { num: 34, name: "Da Zhuang", title: "Great Power", hex: "䷡",
    meaning: "Great power. Perseverance furthers.",
    image: "Thunder above, heaven below. Thunder in heaven above." },
  { num: 35, name: "Jin", title: "Progress", hex: "䷢",
    meaning: "Progress. The powerful prince is honored with horses in large numbers.",
    image: "Fire above, earth below. The sun rises over the earth." },
  { num: 36, name: "Ming Yi", title: "Darkening of the Light", hex: "䷣",
    meaning: "Darkening of the light. Perseverance in adversity furthers.",
    image: "Earth above, fire below. The light has sunk into the earth." },
  { num: 37, name: "Jia Ren", title: "The Family", hex: "䷤",
    meaning: "The family. The perseverance of the woman furthers.",
    image: "Wind above, fire below. Wind comes forth from fire." },
  { num: 38, name: "Kui", title: "Opposition", hex: "䷥",
    meaning: "Opposition. In small matters, good fortune.",
    image: "Fire above, lake below. Fire above, lake below." },
  { num: 39, name: "Jian", title: "Obstruction", hex: "䷦",
    meaning: "Obstruction. The southwest furthers. The northeast does not.",
    image: "Water above, mountain below. Water on the mountain." },
  { num: 40, name: "Xie", title: "Deliverance", hex: "䷧",
    meaning: "Deliverance. The southwest furthers. If there is no longer anything to go.",
    image: "Thunder above, water below. Thunder and rain set in." },
  { num: 41, name: "Sun", title: "Decrease", hex: "䷨",
    meaning: "Decrease combined with sincerity. Supreme good fortune without blame.",
    image: "Mountain above, lake below. A lake at the foot of the mountain." },
  { num: 42, name: "Yi", title: "Increase", hex: "䷩",
    meaning: "Increase. It furthers to undertake something. Crossing the great water.",
    image: "Wind above, thunder below. Wind and thunder." },
  { num: 43, name: "Guai", title: "Breakthrough", hex: "䷪",
    meaning: "Breakthrough. One must resolutely make the matter known at the court.",
    image: "Lake above, heaven below. The lake risen to heaven." },
  { num: 44, name: "Gou", title: "Coming to Meet", hex: "䷫",
    meaning: "Coming to meet. The maiden is powerful. Do not marry such a maiden.",
    image: "Heaven above, wind below. Under heaven, wind." },
  { num: 45, name: "Cui", title: "Gathering Together", hex: "䷬",
    meaning: "Gathering together. Success. The king approaches his temple.",
    image: "Lake above, earth below. Over the earth, the lake." },
  { num: 46, name: "Sheng", title: "Pushing Upward", hex: "䷭",
    meaning: "Pushing upward. Supreme success. One must see the witch.",
    image: "Earth above, wind below. Within the earth, wood grows." },
  { num: 47, name: "Kun", title: "Oppression", hex: "䷮",
    meaning: "Oppression. Success. Perseverance. The witch brings good fortune.",
    image: "Lake above, water below. There is no water in the lake." },
  { num: 48, name: "Jing", title: "The Well", hex: "䷯",
    meaning: "The well. The town may be changed but not the well. No decrease, no increase.",
    image: "Water above, wind below. Wood goes down into the earth." },
  { num: 49, name: "Ge", title: "Revolution", hex: "䷰",
    meaning: "Revolution. On your own day you are believed. Supreme success.",
    image: "Lake above, fire below. Fire in the lake." },
  { num: 50, name: "Ding", title: "The Cauldron", hex: "䷱",
    meaning: "The cauldron. Supreme good fortune. Success.",
    image: "Fire above, wind below. Fire over wood." },
  { num: 51, name: "Zhen", title: "The Arousing", hex: "䷲",
    meaning: "Shock. Success. Shock comes. Laughing words, ha ha!",
    image: "Thunder above, thunder below. Thunder repeated." },
  { num: 52, name: "Gen", title: "Keeping Still", hex: "䷳",
    meaning: "Keeping still. Keep his back still so he no longer feels his body.",
    image: "Mountain above, mountain below. Mountains standing close together." },
  { num: 53, name: "Jian", title: "Development", hex: "䷴",
    meaning: "Development. The maiden is given in marriage. Good fortune.",
    image: "Wind above, mountain below. On the mountain, a tree." },
  { num: 54, name: "Gui Mei", title: "The Marrying Maiden", hex: "䷵",
    meaning: "The marrying maiden. Undertakings bring misfortune.",
    image: "Thunder above, lake below. Thunder over the lake." },
  { num: 55, name: "Feng", title: "Abundance", hex: "䷶",
    meaning: "Abundance. Success. The king attains abundance. Be not sad.",
    image: "Thunder above, fire below. Both thunder and lightning come." },
  { num: 56, name: "Lu", title: "The Wanderer", hex: "䷷",
    meaning: "The wanderer. Success through smallness. Perseverance brings fortune.",
    image: "Fire above, mountain below. Fire on the mountain." },
  { num: 57, name: "Xun", title: "The Gentle", hex: "䷸",
    meaning: "The gentle. Success through small matters. It furthers to have a goal.",
    image: "Wind above, wind below. Winds following one upon the other." },
  { num: 58, name: "Dui", title: "The Joyous", hex: "䷹",
    meaning: "The joyous. Success. Perseverance is favorable.",
    image: "Lake above, lake below. Lakes resting one on the other." },
  { num: 59, name: "Huan", title: "Dispersion", hex: "䷺",
    meaning: "Dispersion. Success. The king approaches his temple.",
    image: "Wind above, water below. Wind blows over water." },
  { num: 60, name: "Jie", title: "Limitation", hex: "䷻",
    meaning: "Limitation. Success. Galling limitation must not be persevered in.",
    image: "Water above, lake below. Water over lake." },
  { num: 61, name: "Zhong Fu", title: "Inner Truth", hex: "䷼",
    meaning: "Inner truth. Pigs and fishes. Good fortune. Crossing the great water.",
    image: "Wind above, lake below. Wind over the lake." },
  { num: 62, name: "Xiao Guo", title: "Small Exceeding", hex: "䷽",
    meaning: "Small exceeding. Success. Perseverance furthers. Small things, yes.",
    image: "Thunder above, mountain below. Thunder on the mountain." },
  { num: 63, name: "Ji Ji", title: "After Completion", hex: "䷾",
    meaning: "After completion. Success in small matters. Perseverance furthers.",
    image: "Water above, fire below. Water over fire." },
  { num: 64, name: "Wei Ji", title: "Before Completion", hex: "䷿",
    meaning: "Before completion. Success. The little fox almost succeeds.",
    image: "Fire above, water below. Fire over water." }
];


export const refreshFrequency = 60000; // Check every minute
export const clickThrough = false;

export const command = `
  CAST=$(cat /tmp/iching-cast 2>/dev/null || date +%Y%m%d)
  echo "$CAST $(date +%M)"
`;

const PRIME = 17; // Appears when minute % 17 === 0

export const render = ({ output }) => {
  if (!output) return null;
  const [castStr, minuteStr] = output.trim().split(' ');
  const minute = parseInt(minuteStr);

  // Visible for 3 minutes each cycle
  if (minute % PRIME >= 3) return null;

  const seed = parseInt(castStr);
  const idx = seed % 64;
  const hex = HEXAGRAMS[idx];

  // Click to cast new reading
  const castUrl = `hammerspoon://iching?action=cast`;

  return (
    <a href={castUrl} style={{...container, textDecoration: 'none', display: 'block'}}>
      <div style={hexagram}>{hex.hex}</div>
      <div style={number}>{hex.num}. {hex.name}</div>
      <div style={title}>{hex.title}</div>
      <div style={meaning}>{hex.meaning}</div>
      <div style={image}>{hex.image}</div>
    </a>
  );
};

const container = {
  position: "fixed",
  top: "280px",
  left: "80px",
  width: "360px",
  padding: "34px",
  backgroundColor: "rgba(26, 26, 46, 0.45)",
  borderRadius: "12px",
  border: "2px solid #aa00e8",
  fontFamily: "SF Pro, Helvetica Neue, sans-serif",
  color: "#ffffff",
  backdropFilter: "blur(10px)"
};

const hexagram = {
  fontSize: "250px",
  textAlign: "center",
  color: "#5cecff",
  marginBottom: "20px"
};

const number = {
  fontSize: "27px",
  color: "#666666",
  textAlign: "center",
  textTransform: "uppercase",
  letterSpacing: "2px",
  fontFamily: "Bradley Hand, cursive"
};

const title = {
  fontSize: "30px",
  fontWeight: "bold",
  color: "#ff0099",
  textAlign: "center",
  marginBottom: "15px"
};

const meaning = {
  fontSize: "26px",
  lineHeight: "1.5",
  color: "#ffffff",
  marginBottom: "12px",
  fontFamily: "Bradley Hand, cursive"
};

const image = {
  fontSize: "28px",
  fontStyle: "italic",
  color: "#888888",
  borderTop: "1px solid #333",
  paddingTop: "10px",
  fontFamily: "Bradley Hand, cursive"
};
