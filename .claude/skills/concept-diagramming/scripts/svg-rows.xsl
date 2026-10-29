<?xml version="1.0" encoding="UTF-8"?>
<!-- Flattens an SVG into tab-separated rows for check-plates.
     P  id  d  marker-end  transforms  class  data-curve
     T  font-family  font-size  x  y  text-anchor  letter-spacing  transforms  class  text
     S  stylesheet text with newlines and tabs replaced by spaces
     transforms is every ancestor-or-self transform attribute, outermost first, joined by ';'. -->
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
                xmlns:xlink="http://www.w3.org/1999/xlink">
  <xsl:output method="text" encoding="UTF-8"/>
  <xsl:variable name="tab" select="'&#9;'"/>
  <xsl:variable name="nl" select="'&#10;'"/>

  <xsl:template match="/">
    <xsl:for-each select="//*[local-name()='path'][not(ancestor::*[local-name()='marker'])]">
      <xsl:value-of select="concat('P', $tab, @id, $tab, normalize-space(@d), $tab, @marker-end, $tab)"/>
      <xsl:call-template name="transforms"/>
      <xsl:value-of select="concat($tab, @class, $tab, @data-curve, $nl)"/>
    </xsl:for-each>
    <xsl:for-each select="//*[local-name()='text']">
      <xsl:value-of select="concat('T', $tab, @font-family, $tab, @font-size, $tab, @x, $tab, @y, $tab, @text-anchor, $tab, @letter-spacing, $tab)"/>
      <xsl:call-template name="transforms"/>
      <xsl:value-of select="concat($tab, @class, $tab, translate(normalize-space(.), $tab, ' '), $nl)"/>
    </xsl:for-each>
    <xsl:for-each select="//*[local-name()='style']">
      <xsl:value-of select="concat('S', $tab, translate(., concat($tab, $nl), '  '), $nl)"/>
    </xsl:for-each>
  </xsl:template>

  <xsl:template name="transforms">
    <xsl:for-each select="ancestor-or-self::*[@transform]">
      <xsl:if test="position() &gt; 1">;</xsl:if>
      <xsl:value-of select="normalize-space(@transform)"/>
    </xsl:for-each>
  </xsl:template>
</xsl:stylesheet>
