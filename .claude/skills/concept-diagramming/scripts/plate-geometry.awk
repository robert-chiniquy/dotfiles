# Reads rows from svg-rows.xsl: the master curve file first (or /dev/null), then one plate.
# Prints ERROR, NOTE, and GLYPH rows, tab-separated, for check-plates.
# Label boxes are estimates (em_width plus letter-spacing), 0.72 em above
# the baseline and 0.05 em below. Paths are sampled; only absolute M, L, H, V, C, Z are read.
BEGIN { FS = "\t"; OFS = "\t" }

# Serif advance estimate in em: capitals and digits 0.66, spaces 0.25, everything else 0.47.
# Runs under LC_ALL=C, so UTF-8 continuation bytes are skipped and each character counts once.
function em_width(s,    i, c, w) {
  w = 0
  for (i = 1; i <= length(s); i++) {
    c = substr(s, i, 1)
    if (c ~ /[\200-\277]/) continue
    w += (c ~ /[A-Z0-9]/) ? 0.66 : (c == " ") ? 0.25 : 0.47
  }
  return w
}

function norm(d) { gsub(/,/, " ", d); gsub(/[A-Za-z]/, " & ", d); gsub(/ +/, " ", d); sub(/^ /, "", d); sub(/ $/, "", d); return d }

# Composes translate() and scale(), outermost first, into x' = SX*x + TX, y' = SY*y + TY.
# Returns 0 when another transform function appears.
function translation(s,    parts, n, i, f, nums, k, name) {
  SX = 1; SY = 1; TX = 0; TY = 0
  if (s == "") return 1
  n = split(s, parts, /\)/)
  for (i = 1; i <= n; i++) {
    f = parts[i]; gsub(/^[ ;]+/, "", f)
    if (f == "") continue
    if (f !~ /^(translate|scale) *\(/) return 0
    name = f; sub(/ *\(.*/, "", name)
    sub(/^[a-z]+ *\( */, "", f); gsub(/,/, " ", f)
    k = split(f, nums, / +/)
    if (name == "translate") { TX += SX * nums[1]; if (k > 1) TY += SY * nums[2] }
    else { SX *= nums[1]; SY *= (k > 1 ? nums[2] : nums[1]) }
  }
  return 1
}

# Class properties from the stylesheet: CSS[class, prop].
function parse_css(s,    rule, name, body, decls, n, i, kv) {
  while (match(s, /\.[A-Za-z0-9_-]+[ ]*\{[^}]*\}/)) {
    rule = substr(s, RSTART, RLENGTH); s = substr(s, RSTART + RLENGTH)
    name = rule; sub(/^\./, "", name); sub(/[ ]*\{.*/, "", name)
    body = rule; sub(/^[^{]*\{/, "", body); sub(/\}$/, "", body)
    n = split(body, decls, /;/)
    for (i = 1; i <= n; i++) {
      if (split(decls[i], kv, /:/) < 2) continue
      gsub(/^ +| +$/, "", kv[1]); gsub(/^ +| +$/, "", kv[2])
      CSS[name, kv[1]] = kv[2]
    }
  }
}

function prop(classes, attr, name,    c, n, i, v) {
  v = ""
  n = split(classes, c, / +/)
  for (i = 1; i <= n; i++) if ((c[i], name) in CSS) v = CSS[c[i], name]
  return attr != "" ? attr : v
}

# Fills PX, PY (NP points) by sampling the path. Returns 0 for unsupported commands.
function sample(d,    t, n, i, cmd, x, y, x1, y1, x2, y2, x3, y3, s, u, sx, sy) {
  NP = 0
  n = split(norm(d), t, / /)
  i = 1; x = 0; y = 0
  while (i <= n) {
    if (t[i] ~ /^[A-Za-z]$/) { cmd = t[i]; i++; if (cmd == "Z") { PX[++NP] = sx; PY[NP] = sy; x = sx; y = sy }; continue }
    if (cmd == "M") { x = t[i] + 0; y = t[i+1] + 0; sx = x; sy = y; PX[++NP] = x; PY[NP] = y; i += 2; cmd = "L"; continue }
    if (cmd == "L" || cmd == "H" || cmd == "V") {
      x1 = x; y1 = y
      if (cmd == "L") { x = t[i] + 0; y = t[i+1] + 0; i += 2 }
      else if (cmd == "H") { x = t[i] + 0; i++ }
      else { y = t[i] + 0; i++ }
      for (s = 1; s <= 12; s++) { PX[++NP] = x1 + (x - x1) * s / 12; PY[NP] = y1 + (y - y1) * s / 12 }
      continue
    }
    if (cmd == "C") {
      x1 = t[i] + 0; y1 = t[i+1] + 0; x2 = t[i+2] + 0; y2 = t[i+3] + 0; x3 = t[i+4] + 0; y3 = t[i+5] + 0; i += 6
      for (s = 1; s <= 24; s++) {
        u = s / 24
        PX[++NP] = (1-u)^3*x + 3*(1-u)^2*u*x1 + 3*(1-u)*u^2*x2 + u^3*x3
        PY[NP] = (1-u)^3*y + 3*(1-u)^2*u*y1 + 3*(1-u)*u^2*y2 + u^3*y3
      }
      x = x3; y = y3
      continue
    }
    return 0
  }
  return 1
}

FNR == NR && $1 == "P" && $2 != "" { MD[$2] = norm($3); MID[norm($3)] = $2; HAVE_MASTER = 1; next }
FNR == NR { next }

$1 == "S" { parse_css($2); next }
$1 == "P" { np++; PD[np] = $3; PM[np] = $4; PT[np] = $5; PC[np] = $6; PK[np] = $7; next }
$1 == "T" { nt++; TF[nt] = $2; TS[nt] = $3; TXX[nt] = $4; TYY[nt] = $5; TA[nt] = $6; TL[nt] = $7; TT[nt] = $8; TC[nt] = $9; TTEXT[nt] = $10; next }

END {
  for (j = 1; j <= nt; j++) {
    fam = prop(TC[j], TF[j], "font-family")
    if (TTEXT[j] != "" && fam != "") print "GLYPH", fam, TTEXT[j]
    size = prop(TC[j], TS[j], "font-size"); sub(/px$/, "", size); size += 0; if (size == 0) size = 16
    ls = prop(TC[j], TL[j], "letter-spacing"); sub(/px$/, "", ls); ls += 0
    anchor = prop(TC[j], TA[j], "text-anchor")
    if (!translation(TT[j])) { print "NOTE", "label \"" TTEXT[j] "\" has a rotate, skew, or matrix transform; overlap not checked"; BOX[j] = 0; continue }
    w = em_width(TTEXT[j]) * size * SX + (utf8_length(TTEXT[j]) - 1) * ls * SX
    x0 = TXX[j] * SX + TX; if (anchor == "middle") x0 -= w / 2; else if (anchor == "end") x0 -= w
    y0 = TYY[j] * SY + TY
    BX0[j] = x0; BX1[j] = x0 + w; BY0[j] = y0 - 0.72 * size * SY; BY1[j] = y0 + 0.05 * size * SY
    BM[j] = 0.1 * size * SY; BOX[j] = 1
  }
  for (i = 1; i <= np; i++) {
    d = norm(PD[i]); short = substr(PD[i], 1, 32)
    rule = (d !~ /[^MHV0-9. -]/)
    if (HAVE_MASTER && !rule) {
      if (PK[i] != "") {
        if (!(PK[i] in MD)) print "ERROR", "path data-curve=" PK[i] " names no curve in the master file"
        else if (MD[PK[i]] != d) print "ERROR", "path data-curve=" PK[i] " differs from the master curve: " short
      } else if (d in MID) print "NOTE", "path copies master curve " MID[d] " without data-curve: " short
      else print "NOTE", "path is not in the master curve set: " short
    }
    if (!translation(PT[i])) { print "NOTE", "path has a rotate, skew, or matrix transform; overlap not checked: " short; continue }
    if (!sample(PD[i])) { print "NOTE", "path uses commands this check does not read; overlap not checked: " short; continue }
    arrow = (prop(PC[i], PM[i], "marker-end") != "")
    for (j = 1; j <= nt; j++) {
      if (!BOX[j]) continue
      for (p = 1; p <= NP; p++) {
        m = (arrow && p == NP) ? BM[j] : 0
        x = PX[p] * SX + TX; y = PY[p] * SY + TY
        if (x >= BX0[j] - m && x <= BX1[j] + m && y >= BY0[j] - m && y <= BY1[j] + m) {
          what = (arrow && p == NP) ? "arrowhead lands on" : "path crosses"
          printf "ERROR\t%s label \"%s\" near (%d,%d): %s\n", what, TTEXT[j], x, y, short
          break
        }
      }
    }
  }
}

function utf8_length(s,    t) { t = s; gsub(/[\200-\277]/, "", t); return length(t) }
